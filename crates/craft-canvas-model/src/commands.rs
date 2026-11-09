//! Atomic commands preserve untouched raw records and merge only explicit fields.
use crate::{
    CanvasDialect, CanvasDocument, CanvasError, CanvasLimits, ConnectionOperation,
    ConnectionRequest, ConnectorEndpoint, ConnectorEndpointKind, ContentIdentity, EdgeRecord,
    ExtensionStatus, GroupBehavior, LabelTarget, NodeKind, NodeRecord, PortBinding, PortDefinition,
    PortEndpoint, ReconnectPolicy, Result, WorldPoint, WorldRect,
    connections::validate_connection_against,
    document::{RawMap, raw, valid_id},
    json_guard::check_json,
};
use serde_json::value::RawValue;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Selected stable identities, independent of transient graph indices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    /// Selected node IDs.
    pub nodes: BTreeSet<String>,
    /// Selected edge IDs.
    pub edges: BTreeSet<String>,
}
impl Selection {
    /// Keep only identities still present after a host acknowledgement.
    pub fn retain_existing(&mut self, document: &CanvasDocument) {
        self.nodes.retain(|id| document.node(id).is_some());
        self.edges.retain(|id| document.edge(id).is_some());
    }
}
/// Explicit caller-allocated mapping for duplication; no library-generated ambient IDs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdMapping {
    /// Original node ID to fresh node ID.
    pub nodes: BTreeMap<String, String>,
    /// Original edge ID to fresh edge ID.
    pub edges: BTreeMap<String, String>,
}
/// Stable selection relationships produced by a complete atomic change.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionMap {
    /// Replaced/duplicated node identities.
    pub nodes: BTreeMap<String, String>,
    /// Replaced/duplicated edge identities.
    pub edges: BTreeMap<String, String>,
    /// Newly inserted node identities.
    pub added_nodes: BTreeSet<String>,
    /// Newly inserted edge identities.
    pub added_edges: BTreeSet<String>,
    /// Deleted node identities.
    pub removed_nodes: BTreeSet<String>,
    /// Deleted edge identities, including incident connectors.
    pub removed_edges: BTreeSet<String>,
}
impl SelectionMap {
    pub(crate) fn inverse(&self) -> Self {
        Self {
            nodes: self
                .nodes
                .iter()
                .map(|(a, b)| (b.clone(), a.clone()))
                .collect(),
            edges: self
                .edges
                .iter()
                .map(|(a, b)| (b.clone(), a.clone()))
                .collect(),
            added_nodes: self.removed_nodes.clone(),
            added_edges: self.removed_edges.clone(),
            removed_nodes: self.added_nodes.clone(),
            removed_edges: self.added_edges.clone(),
        }
    }
}
/// One explicit known-field update. Nested object paths preserve unrelated children.
#[derive(Debug)]
pub struct FieldUpdate {
    path: Vec<String>,
    value: Option<Box<RawValue>>,
}
impl FieldUpdate {
    /// Set one field to bounded valid JSON, rejecting duplicate keys before interpretation.
    pub fn set(path: Vec<String>, json: &str, limits: &CanvasLimits) -> Result<Self> {
        limits.validate()?;
        check_json(json.as_bytes(), limits)?;
        Self::validate_path(&path, limits)?;
        Ok(Self {
            path,
            value: Some(serde_json::from_str(json)?),
        })
    }
    /// Remove one explicit field without reconstructing its siblings.
    pub fn remove(path: Vec<String>, limits: &CanvasLimits) -> Result<Self> {
        Self::validate_path(&path, limits)?;
        Ok(Self { path, value: None })
    }
    /// Exact object path.
    pub fn path(&self) -> &[String] {
        &self.path
    }
    /// New value, or None for removal.
    pub fn value(&self) -> Option<&RawValue> {
        self.value.as_deref()
    }
    fn validate_path(path: &[String], limits: &CanvasLimits) -> Result<()> {
        if path.is_empty()
            || path.len() > limits.max_depth
            || path
                .iter()
                .any(|key| key.is_empty() || key.len() > limits.max_bytes)
        {
            return Err(CanvasError::Invalid("invalid field path".into()));
        }
        Ok(())
    }
}
/// A complete generic graph action. Host resource/application validation remains mandatory.
#[derive(Debug)]
pub enum CanvasCommand {
    /// Insert a validated node at a canonical array ordinal.
    InsertNode {
        /// Complete preserved node.
        node: NodeRecord,
        /// Ordinal from 0 through current count.
        index: usize,
    },
    /// Insert a validated connector at a canonical array ordinal.
    InsertEdge {
        /// Complete preserved edge.
        edge: EdgeRecord,
        /// Ordinal from 0 through current count.
        index: usize,
    },
    /// Insert a connector while retaining the actual initiating endpoint for validation.
    ConnectPorts {
        /// Complete connector with a caller-allocated stable identity.
        edge: EdgeRecord,
        /// Canonical array insertion ordinal.
        index: usize,
        /// Initiating end after normalization into source and target endpoints.
        start: ConnectorEndpointKind,
    },
    /// Replace a node's known port catalogue, preserving matching-port unknown members.
    SetPorts {
        /// Stable node identity.
        node_id: String,
        /// Complete replacement catalogue. Incident bindings validate against the final transaction.
        ports: Vec<PortDefinition>,
    },
    /// Reconnect a named or legacy binding without replacing the edge identity.
    ReconnectPort {
        /// Stable edge identity.
        edge_id: String,
        /// End being changed; the stationary opposite end is the initiating port.
        endpoint: ConnectorEndpointKind,
        /// Exact previous semantic binding.
        expected: PortEndpoint,
        /// Explicit replacement binding.
        replacement: PortEndpoint,
    },
    /// Update future reconnection policy; this cannot authorize a reconnect of the old edge.
    SetReconnectPolicy {
        /// Stable edge identity.
        edge_id: String,
        /// Policy for subsequent transactions.
        policy: ReconnectPolicy,
    },
    /// Change exact geometry with independent move/resize policy checks.
    SetGeometry {
        /// Stable node ID.
        node_id: String,
        /// Expected current rectangle.
        expected: WorldRect,
        /// Requested replacement rectangle.
        replacement: WorldRect,
    },
    /// Move selected nodes and spatial-group members exactly once.
    Translate {
        /// Explicit stable node IDs.
        node_ids: Vec<String>,
        /// Finite world displacement.
        delta: WorldPoint,
    },
    /// Replace a text card's complete Markdown source as one editing action.
    EditText {
        /// Stable text-card ID.
        node_id: String,
        /// Exact old text including Unicode and newlines.
        expected: String,
        /// New complete Markdown source.
        text: String,
    },
    /// Merge explicit nonidentity/nongeometry fields, including preserved extension children.
    SetNodeFields {
        /// Stable node ID.
        node_id: String,
        /// Deliberate known-field updates.
        fields: Vec<FieldUpdate>,
    },
    /// Merge explicit connector fields, preserving all others.
    SetEdgeFields {
        /// Stable connector ID.
        edge_id: String,
        /// Deliberate updates; ID cannot change.
        fields: Vec<FieldUpdate>,
    },
    /// Reconnect one exact endpoint on the same preserved connector identity.
    ReconnectEdge {
        /// Stable existing edge ID.
        edge_id: String,
        /// Endpoint being changed.
        endpoint: ConnectorEndpointKind,
        /// Exact current endpoint, including whether its side was omitted.
        expected: ConnectorEndpoint,
        /// New explicit board-local endpoint.
        replacement: ConnectorEndpoint,
    },
    /// Change a known group/edge label as one deliberate source-compatible action.
    EditLabel {
        /// Stable group or connector target.
        target: LabelTarget,
        /// Exact old displayed label.
        expected: String,
        /// New input; source UI whitespace trimming and empty-label removal are applied.
        label: String,
    },
    /// Remove only selected nodes and selected/incident edges; group members remain.
    RemoveSelection(Selection),
    /// Duplicate selected nodes, enclosed group members and internal connectors.
    Duplicate {
        /// Explicit node IDs before group expansion.
        node_ids: Vec<String>,
        /// Exact complete fresh identity allocation.
        ids: IdMapping,
        /// Finite world offset.
        delta: WorldPoint,
    },
    /// Set the exact complete node permutation; host determines background-band policy.
    ReorderNodes {
        /// Every current node ID exactly once.
        order: Vec<String>,
    },
}
/// Data-only request at a non-deserializable exact session identity.
#[derive(Debug)]
pub struct CanvasTransaction {
    pub(crate) base: ContentIdentity,
    pub(crate) commands: Vec<CanvasCommand>,
}
impl CanvasTransaction {
    /// Build a request from a token obtained from the current session.
    pub fn new(base: ContentIdentity, commands: Vec<CanvasCommand>) -> Self {
        Self { base, commands }
    }
    /// Guarded base; this token alone does not grant project authorization.
    pub fn base(&self) -> ContentIdentity {
        self.base
    }
    /// Borrow commands for host policy/preview inspection.
    pub fn commands(&self) -> &[CanvasCommand] {
        &self.commands
    }
}
/// Charged work and cooperative cancellation for potentially large graph operations.
pub struct WorkControl<'a> {
    remaining: usize,
    cancelled: Option<&'a AtomicBool>,
}
impl<'a> WorkControl<'a> {
    /// Create an operation-owned work budget. Hosts run expensive preparation off the UI thread.
    pub fn new(units: usize, cancelled: Option<&'a AtomicBool>) -> Self {
        Self {
            remaining: units,
            cancelled,
        }
    }
    /// Remaining charged units.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    /// Consume shared work units, refusing cancellation or budget exhaustion.
    /// Native adapters can charge their own traversal against the same bounded operation.
    pub fn charge(&mut self, units: usize) -> Result<()> {
        if self
            .cancelled
            .is_some_and(|flag| flag.load(Ordering::Acquire))
        {
            return Err(CanvasError::Cancelled);
        }
        self.remaining = self
            .remaining
            .checked_sub(units)
            .ok_or(CanvasError::WorkLimit)?;
        Ok(())
    }
}
pub(crate) fn execute(
    document: &CanvasDocument,
    commands: Vec<CanvasCommand>,
    dialect: &dyn CanvasDialect,
    limits: &CanvasLimits,
    work: &mut WorkControl<'_>,
) -> Result<(CanvasDocument, SelectionMap)> {
    if commands.len() > limits.max_commands {
        return Err(CanvasError::Limit("transaction commands"));
    }
    let mut nodes = document.nodes().to_vec();
    let mut edges = document.edges().to_vec();
    let mut mapping = SelectionMap::default();
    let mut authored_edges = HashMap::new();
    let mut copied_edges: HashMap<String, Arc<EdgeRecord>> = HashMap::new();
    let mut resolved_ports = HashMap::new();
    for edge in document.edges() {
        work.charge(1)?;
        resolved_ports.insert(
            edge.id().to_owned(),
            [
                named_port_resolves(document, edge, ConnectorEndpointKind::From),
                named_port_resolves(document, edge, ConnectorEndpointKind::To),
            ],
        );
    }
    for command in commands {
        work.charge(1)?;
        match command {
            CanvasCommand::InsertNode { node, index } => {
                if index > nodes.len() || nodes.len() >= limits.max_nodes {
                    return Err(CanvasError::Limit("node insertion"));
                }
                if nodes.iter().any(|old| old.id() == node.id()) {
                    return Err(CanvasError::Invalid("duplicate node ID".into()));
                }
                mapping.added_nodes.insert(node.id().into());
                nodes.insert(index, Arc::new(node));
            }
            CanvasCommand::InsertEdge { edge, index } => {
                authored_edges.insert(edge.id().to_owned(), ConnectorEndpointKind::From);
                insert_edge(&mut edges, edge, index, &mut mapping, limits)?;
            }
            CanvasCommand::ConnectPorts { edge, index, start } => {
                authored_edges.insert(edge.id().to_owned(), start);
                insert_edge(&mut edges, edge, index, &mut mapping, limits)?;
            }
            CanvasCommand::SetPorts { node_id, ports } => {
                let at = node_position(&nodes, &node_id, work)?;
                work.charge(ports.len() + nodes[at].ports().len())?;
                let next = nodes[at].with_ports(&ports, limits)?;
                if next.raw_json() != nodes[at].raw_json() {
                    nodes[at] = Arc::new(next);
                }
            }
            CanvasCommand::SetReconnectPolicy { edge_id, policy } => {
                let at = edge_position(&edges, &edge_id, work)?;
                let next = edges[at].with_reconnect_policy(policy, limits)?;
                if next.raw_json() != edges[at].raw_json() {
                    edges[at] = Arc::new(next);
                }
            }
            CanvasCommand::ReconnectPort {
                edge_id,
                endpoint,
                expected,
                replacement,
            } => {
                let at = edge_position(&edges, &edge_id, work)?;
                let actual = edges[at].endpoint(endpoint)?;
                if actual != expected {
                    return Err(CanvasError::StaleBase);
                }
                if actual != replacement {
                    edges[at] =
                        Arc::new(edges[at].with_port_endpoint(endpoint, &replacement, limits)?);
                }
            }
            CanvasCommand::SetGeometry {
                node_id,
                expected,
                replacement,
            } => {
                let at = node_position(&nodes, &node_id, work)?;
                let node = &nodes[at];
                if node.bounds() != expected {
                    return Err(CanvasError::StaleBase);
                }
                if replacement == expected {
                    continue;
                }
                replacement.validate(limits)?;
                let policy = dialect.node_policy(node);
                if ((expected.x != replacement.x || expected.y != replacement.y) && !policy.movable)
                    || ((expected.width != replacement.width
                        || expected.height != replacement.height)
                        && !policy.resizable)
                {
                    return Err(CanvasError::NotEditable(node_id));
                }
                work.charge(node.ports().len().saturating_mul(2))?;
                nodes[at] = Arc::new(with_rect(node, replacement, limits)?);
            }
            CanvasCommand::Translate { node_ids, delta } => {
                WorldPoint::new(delta.x, delta.y)?;
                let targets = expanded_selection(&nodes, &node_ids, dialect, work)?;
                for node in &mut nodes {
                    work.charge(1)?;
                    if targets.contains(node.id()) {
                        let replacement = node.bounds().translated(delta).validate(limits)?;
                        if replacement != node.bounds() {
                            work.charge(node.ports().len().saturating_mul(2))?;
                            *node = Arc::new(with_rect(node, replacement, limits)?);
                        }
                    }
                }
            }
            CanvasCommand::EditText {
                node_id,
                expected,
                text,
            } => {
                let at = node_position(&nodes, &node_id, work)?;
                let node = &nodes[at];
                if !matches!(node.kind(), NodeKind::Text) {
                    return Err(CanvasError::NotEditable(node_id));
                }
                if node.body() != expected {
                    return Err(CanvasError::StaleBase);
                }
                if node.body() == text {
                    continue;
                }
                let mut fields = node.fields().clone();
                fields.insert("text".into(), raw(&text)?);
                work.charge(node.ports().len().saturating_mul(2))?;
                nodes[at] = Arc::new(node.with_fields(fields, limits)?);
            }
            CanvasCommand::SetNodeFields { node_id, fields } => {
                let at = node_position(&nodes, &node_id, work)?;
                let node = &nodes[at];
                let mut next = node.fields().clone();
                apply_fields(
                    &mut next,
                    fields,
                    &["id", "type", "x", "y", "width", "height", "text"],
                    limits,
                    work,
                )?;
                if fields_equal(&next, node.fields()) {
                    continue;
                }
                guard_preserved_extension(
                    node.port_extension_status(),
                    node.field("craftCanvas"),
                    next.get("craftCanvas").map(Box::as_ref),
                )?;
                let next = node.with_fields(next, limits)?;
                work.charge(node.ports().len() + next.ports().len())?;
                guard_extension_change(
                    node.port_extension_status(),
                    next.port_extension_status(),
                    node.field("craftCanvas"),
                    next.field("craftCanvas"),
                )?;
                nodes[at] = Arc::new(next);
            }
            CanvasCommand::SetEdgeFields { edge_id, fields } => {
                work.charge(edges.len())?;
                let at = edges
                    .iter()
                    .position(|edge| edge.id() == edge_id)
                    .ok_or(CanvasError::NotFound(edge_id))?;
                let edge = &edges[at];
                let mut next = edge.fields().clone();
                apply_fields(&mut next, fields, &["id"], limits, work)?;
                if fields_equal(&next, edge.fields()) {
                    continue;
                }
                guard_preserved_extension(
                    edge.port_extension_status(),
                    edge.field("craftCanvas"),
                    next.get("craftCanvas").map(Box::as_ref),
                )?;
                let next = edge.with_fields(next, limits)?;
                guard_extension_change(
                    edge.port_extension_status(),
                    next.port_extension_status(),
                    edge.field("craftCanvas"),
                    next.field("craftCanvas"),
                )?;
                edges[at] = Arc::new(next);
            }
            CanvasCommand::ReconnectEdge {
                edge_id,
                endpoint,
                expected,
                replacement,
            } => {
                work.charge(edges.len())?;
                let at = edges
                    .iter()
                    .position(|edge| edge.id() == edge_id)
                    .ok_or_else(|| CanvasError::NotFound(edge_id.clone()))?;
                let edge = &edges[at];
                let actual = edge.endpoint(endpoint)?;
                let PortBinding::Legacy(side) = actual.binding else {
                    return Err(CanvasError::Invalid(
                        "use ReconnectPort for a named endpoint".into(),
                    ));
                };
                if actual.node_id != expected.node_id || side != expected.side {
                    return Err(CanvasError::StaleBase);
                }
                let replacement = PortEndpoint {
                    node_id: replacement.node_id,
                    binding: PortBinding::Legacy(replacement.side),
                };
                if actual != replacement {
                    edges[at] =
                        Arc::new(edge.with_port_endpoint(endpoint, &replacement, limits)?);
                }
            }
            CanvasCommand::EditLabel {
                target,
                expected,
                label,
            } => {
                let trimmed=label.trim_matches(|ch|matches!(ch,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}'));
                if trimmed.chars().count() > 500 {
                    return Err(CanvasError::Limit("label characters"));
                }
                match target {
                    LabelTarget::Node(id) => {
                        let at = node_position(&nodes, &id, work)?;
                        let node = &nodes[at];
                        if !matches!(node.kind(), NodeKind::Group) {
                            return Err(CanvasError::NotEditable(id));
                        }
                        if node.body() != expected {
                            return Err(CanvasError::StaleBase);
                        }
                        if node.body() == trimmed {
                            continue;
                        }
                        let mut fields = node.fields().clone();
                        if trimmed.is_empty() {
                            fields.remove("label");
                        } else {
                            fields.insert("label".into(), raw(&trimmed)?);
                        }
                        work.charge(node.ports().len().saturating_mul(2))?;
                        nodes[at] = Arc::new(node.with_fields(fields, limits)?);
                    }
                    LabelTarget::Edge(id) => {
                        work.charge(edges.len())?;
                        let at = edges
                            .iter()
                            .position(|edge| edge.id() == id)
                            .ok_or(CanvasError::NotFound(id))?;
                        let edge = &edges[at];
                        if edge.label() != expected {
                            return Err(CanvasError::StaleBase);
                        }
                        if edge.label() == trimmed {
                            continue;
                        }
                        let mut fields = edge.fields().clone();
                        if trimmed.is_empty() {
                            fields.remove("label");
                        } else {
                            fields.insert("label".into(), raw(&trimmed)?);
                        }
                        edges[at] = Arc::new(edge.with_fields(fields, limits)?);
                    }
                }
            }
            CanvasCommand::RemoveSelection(selection) => {
                work.charge(
                    nodes.len() + edges.len() + selection.nodes.len() + selection.edges.len(),
                )?;
                let node_ids: HashSet<_> = nodes.iter().map(|node| node.id()).collect();
                let edge_ids: HashSet<_> = edges.iter().map(|edge| edge.id()).collect();
                for id in &selection.nodes {
                    if !node_ids.contains(id.as_str()) {
                        return Err(CanvasError::NotFound(id.clone()));
                    }
                }
                for id in &selection.edges {
                    if !edge_ids.contains(id.as_str()) {
                        return Err(CanvasError::NotFound(id.clone()));
                    }
                }
                work.charge(nodes.len() + edges.len())?;
                nodes.retain(|node| {
                    if selection.nodes.contains(node.id()) {
                        mapping.removed_nodes.insert(node.id().into());
                        false
                    } else {
                        true
                    }
                });
                edges.retain(|edge| {
                    if selection.edges.contains(edge.id())
                        || selection.nodes.contains(edge.from_node())
                        || selection.nodes.contains(edge.to_node())
                    {
                        mapping.removed_edges.insert(edge.id().into());
                        copied_edges.remove(edge.id());
                        authored_edges.remove(edge.id());
                        if document.edge(edge.id()).is_none() {
                            resolved_ports.remove(edge.id());
                        }
                        false
                    } else {
                        true
                    }
                });
            }
            CanvasCommand::Duplicate {
                node_ids,
                ids,
                delta,
            } => {
                WorldPoint::new(delta.x, delta.y)?;
                let targets = expanded_selection(&nodes, &node_ids, dialect, work)?;
                work.charge(edges.len())?;
                let internal: BTreeSet<_> = edges
                    .iter()
                    .filter(|edge| {
                        targets.contains(edge.from_node()) && targets.contains(edge.to_node())
                    })
                    .map(|edge| edge.id().to_owned())
                    .collect();
                if targets != ids.nodes.keys().cloned().collect()
                    || internal != ids.edges.keys().cloned().collect()
                {
                    return Err(CanvasError::Invalid(
                        "duplication ID allocation does not match exact expanded graph".into(),
                    ));
                }
                if nodes
                    .len()
                    .checked_add(targets.len())
                    .is_none_or(|n| n > limits.max_nodes)
                    || edges
                        .len()
                        .checked_add(internal.len())
                        .is_none_or(|n| n > limits.max_edges)
                {
                    return Err(CanvasError::Limit("duplicated graph size"));
                }
                work.charge(nodes.len() + edges.len() + ids.nodes.len() + ids.edges.len())?;
                let existing: HashSet<_> = nodes
                    .iter()
                    .map(|node| node.id())
                    .chain(edges.iter().map(|edge| edge.id()))
                    .collect();
                let mut fresh = HashSet::new();
                for id in ids.nodes.values().chain(ids.edges.values()) {
                    valid_id(id, limits)?;
                    if !fresh.insert(id.as_str()) || existing.contains(id.as_str()) {
                        return Err(CanvasError::Invalid(
                            "duplicated identity is not fresh".into(),
                        ));
                    }
                }
                let mut added_nodes = Vec::with_capacity(targets.len());
                for node in &nodes {
                    work.charge(1)?;
                    if let Some(id) = ids.nodes.get(node.id()) {
                        let mut fields = node.fields().clone();
                        fields.insert("id".into(), raw(id)?);
                        let moved = node.bounds().translated(delta).validate(limits)?;
                        put_rect(&mut fields, moved)?;
                        work.charge(node.ports().len().saturating_mul(2))?;
                        added_nodes.push(Arc::new(node.with_fields(fields, limits)?));
                    }
                }
                let mut added_edges = Vec::with_capacity(internal.len());
                for edge in &edges {
                    work.charge(1)?;
                    if let Some(id) = ids.edges.get(edge.id()) {
                        let mut fields = edge.fields().clone();
                        fields.insert("id".into(), raw(id)?);
                        fields.insert(
                            "fromNode".into(),
                            raw(ids.nodes.get(edge.from_node()).ok_or_else(|| {
                                CanvasError::Invalid("missing duplicated source".into())
                            })?)?,
                        );
                        fields.insert(
                            "toNode".into(),
                            raw(ids.nodes.get(edge.to_node()).ok_or_else(|| {
                                CanvasError::Invalid("missing duplicated target".into())
                            })?)?,
                        );
                        let authored = authored_edges.get(edge.id()).copied();
                        let changed = match document
                            .edge(edge.id())
                            .or_else(|| copied_edges.get(edge.id()).map(Arc::as_ref))
                        {
                            None => true,
                            Some(before) => {
                                endpoint_changed(before, edge, ConnectorEndpointKind::From)?
                                    || endpoint_changed(before, edge, ConnectorEndpointKind::To)?
                            }
                        };
                        if authored.is_some() || changed {
                            authored_edges.insert(
                                id.clone(),
                                authored.unwrap_or(ConnectorEndpointKind::From),
                            );
                        }
                        let copy = Arc::new(edge.with_fields(fields, limits)?);
                        copied_edges.insert(id.clone(), Arc::clone(&copy));
                        if let Some(resolved) = resolved_ports.get(edge.id()).copied() {
                            resolved_ports.insert(id.clone(), resolved);
                        }
                        added_edges.push(copy);
                    }
                }
                mapping.added_nodes.extend(ids.nodes.values().cloned());
                mapping.added_edges.extend(ids.edges.values().cloned());
                nodes.extend(added_nodes);
                edges.extend(added_edges);
                mapping.nodes.extend(ids.nodes);
                mapping.edges.extend(ids.edges);
            }
            CanvasCommand::ReorderNodes { order } => {
                if order.len() != nodes.len() {
                    return Err(CanvasError::Invalid(
                        "node order must be a complete permutation".into(),
                    ));
                }
                let lookup: BTreeMap<_, _> =
                    nodes.iter().map(|node| (node.id(), node.clone())).collect();
                let mut seen = HashSet::new();
                let mut next = Vec::with_capacity(order.len());
                for id in &order {
                    work.charge(1)?;
                    if !seen.insert(id.as_str()) {
                        return Err(CanvasError::Invalid("duplicate ID in node order".into()));
                    }
                    next.push(
                        lookup
                            .get(id.as_str())
                            .ok_or_else(|| CanvasError::NotFound(id.clone()))?
                            .clone(),
                    );
                }
                nodes = next;
            }
        }
    }
    work.charge(nodes.len() + edges.len())?;
    let unchanged = nodes.len() == document.nodes().len()
        && edges.len() == document.edges().len()
        && nodes
            .iter()
            .zip(document.nodes())
            .all(|(a, b)| Arc::ptr_eq(a, b))
        && edges
            .iter()
            .zip(document.edges())
            .all(|(a, b)| Arc::ptr_eq(a, b));
    let after = if unchanged {
        document.clone()
    } else {
        CanvasDocument::from_parts(document.data.top.clone(), nodes, edges, limits)?
    };
    validate_final_connections(
        FinalConnectionContext {
            original: document,
            candidate: &after,
            authored: &authored_edges,
            copies: &copied_edges,
        },
        &mapping,
        &resolved_ports,
        dialect,
        work,
    )?;
    dialect.validate(&after)?;
    Ok((after, mapping))
}
fn insert_edge(
    edges: &mut Vec<Arc<EdgeRecord>>,
    edge: EdgeRecord,
    index: usize,
    mapping: &mut SelectionMap,
    limits: &CanvasLimits,
) -> Result<()> {
    if index > edges.len() || edges.len() >= limits.max_edges {
        return Err(CanvasError::Limit("edge insertion"));
    }
    if edges.iter().any(|old| old.id() == edge.id()) {
        return Err(CanvasError::Invalid("duplicate edge ID".into()));
    }
    mapping.added_edges.insert(edge.id().into());
    edges.insert(index, Arc::new(edge));
    Ok(())
}
fn edge_position(edges: &[Arc<EdgeRecord>], id: &str, work: &mut WorkControl<'_>) -> Result<usize> {
    work.charge(edges.len())?;
    edges
        .iter()
        .position(|edge| edge.id() == id)
        .ok_or_else(|| CanvasError::NotFound(id.into()))
}
fn guard_preserved_extension(
    status: ExtensionStatus,
    old_raw: Option<&RawValue>,
    new_raw: Option<&RawValue>,
) -> Result<()> {
    if matches!(status, ExtensionStatus::Opaque | ExtensionStatus::Malformed)
        && old_raw.map(RawValue::get) != new_raw.map(RawValue::get)
    {
        return Err(CanvasError::Invalid(
            "cannot overwrite a preserved opaque or malformed craftCanvas extension".into(),
        ));
    }
    Ok(())
}
fn guard_extension_change(
    before: ExtensionStatus,
    after: ExtensionStatus,
    old_raw: Option<&RawValue>,
    new_raw: Option<&RawValue>,
) -> Result<()> {
    if old_raw.map(RawValue::get) == new_raw.map(RawValue::get) {
        return Ok(());
    }
    if matches!(before, ExtensionStatus::Opaque | ExtensionStatus::Malformed)
        || matches!(after, ExtensionStatus::Opaque | ExtensionStatus::Malformed)
    {
        return Err(CanvasError::Invalid(
            "unsupported or malformed craftCanvas extension; preserve it before editing ports"
                .into(),
        ));
    }
    Ok(())
}
fn endpoint_changed(
    before: &EdgeRecord,
    after: &EdgeRecord,
    end: ConnectorEndpointKind,
) -> Result<bool> {
    match (before.endpoint(end), after.endpoint(end)) {
        (Ok(old), Ok(new)) => Ok(old != new),
        (old, new) => {
            let same_legacy = match end {
                ConnectorEndpointKind::From => {
                    before.from_node() == after.from_node()
                        && before.from_side() == after.from_side()
                }
                ConnectorEndpointKind::To => {
                    before.to_node() == after.to_node() && before.to_side() == after.to_side()
                }
            };
            if same_legacy
                && before.field("craftCanvas").map(RawValue::get)
                    == after.field("craftCanvas").map(RawValue::get)
            {
                Ok(false)
            } else {
                old?;
                new?;
                Ok(true)
            }
        }
    }
}
fn named_port_resolves(
    document: &CanvasDocument,
    edge: &EdgeRecord,
    end: ConnectorEndpointKind,
) -> bool {
    let Ok(binding) = edge.endpoint(end) else {
        return false;
    };
    let PortBinding::Named(ref key) = binding.binding else {
        return false;
    };
    document
        .node(&binding.node_id)
        .and_then(|node| node.port(key))
        .is_some()
}
// Classification keeps canonical reconnect policy distinct from copying preserved imports.
struct FinalConnectionContext<'a> {
    original: &'a CanvasDocument,
    candidate: &'a CanvasDocument,
    authored: &'a HashMap<String, ConnectorEndpointKind>,
    copies: &'a HashMap<String, Arc<EdgeRecord>>,
}
impl FinalConnectionContext<'_> {
    fn request(&self, edge: &EdgeRecord) -> Result<Option<ConnectionRequest>> {
        let create = |start| {
            (
                start,
                ConnectionOperation::Create {
                    edge_id: edge.id().into(),
                },
            )
        };
        let action = if let Some(before) = self.original.edge(edge.id()) {
            let from = endpoint_changed(before, edge, ConnectorEndpointKind::From)?;
            let to = endpoint_changed(before, edge, ConnectorEndpointKind::To)?;
            match (from, to) {
                (true, true) => Some((
                    ConnectorEndpointKind::From,
                    ConnectionOperation::ReconnectBoth {
                        edge_id: edge.id().into(),
                    },
                )),
                (true, false) => Some((
                    ConnectorEndpointKind::To,
                    ConnectionOperation::Reconnect {
                        edge_id: edge.id().into(),
                        endpoint: ConnectorEndpointKind::From,
                    },
                )),
                (false, true) => Some((
                    ConnectorEndpointKind::From,
                    ConnectionOperation::Reconnect {
                        edge_id: edge.id().into(),
                        endpoint: ConnectorEndpointKind::To,
                    },
                )),
                (false, false) => self.authored.get(edge.id()).copied().map(create),
            }
        } else if let Some(start) = self.authored.get(edge.id()) {
            Some(create(*start))
        } else if let Some(before) = self.copies.get(edge.id()) {
            let from = endpoint_changed(before, edge, ConnectorEndpointKind::From)?;
            let to = endpoint_changed(before, edge, ConnectorEndpointKind::To)?;
            match (from, to) {
                (false, false) => None,
                (true, false) => Some(create(ConnectorEndpointKind::To)),
                _ => Some(create(ConnectorEndpointKind::From)),
            }
        } else {
            None
        };
        action
            .map(|(start, operation)| {
                Ok(ConnectionRequest {
                    from: edge.endpoint(ConnectorEndpointKind::From)?,
                    to: edge.endpoint(ConnectorEndpointKind::To)?,
                    start,
                    operation,
                })
            })
            .transpose()
    }
}
fn validate_final_connections(
    context: FinalConnectionContext<'_>,
    mapping: &SelectionMap,
    resolved_ports: &HashMap<String, [bool; 2]>,
    dialect: &dyn CanvasDialect,
    work: &mut WorkControl<'_>,
) -> Result<()> {
    let original = context.original;
    let candidate = context.candidate;
    for edge in candidate.edges() {
        work.charge(1)?;
        let request = context.request(edge)?;
        if request.is_some() || mapping.added_edges.contains(edge.id()) {
            // Retain established can_connect checks for preservation copies as well.
            for id in [edge.from_node(), edge.to_node()] {
                let node = context
                    .candidate
                    .node(id)
                    .ok_or_else(|| CanvasError::NotFound(id.into()))?;
                if !dialect.can_connect(node) {
                    return Err(CanvasError::NotEditable(id.into()));
                }
            }
        }
        if let Some(request) = request {
            validate_connection_against(original, candidate, &request, dialect)?;
        }
        if let Some(resolved) = resolved_ports.get(edge.id()) {
            for (end, was_resolved) in [ConnectorEndpointKind::From, ConnectorEndpointKind::To]
                .into_iter()
                .zip(resolved)
            {
                if !was_resolved {
                    continue;
                }
                let binding = edge.endpoint(end)?;
                if let PortBinding::Named(ref key) = binding.binding
                    && candidate
                        .node(&binding.node_id)
                        .and_then(|node| node.port(key))
                        .is_none()
                {
                    return Err(CanvasError::Invalid(format!(
                        "port {} remains connected by edge {}; reconnect or remove the edge in the same transaction",
                        key.id,
                        edge.id()
                    )));
                }
            }
        }
    }
    Ok(())
}
fn node_position(nodes: &[Arc<NodeRecord>], id: &str, work: &mut WorkControl<'_>) -> Result<usize> {
    work.charge(nodes.len())?;
    nodes
        .iter()
        .position(|node| node.id() == id)
        .ok_or_else(|| CanvasError::NotFound(id.into()))
}
pub(crate) fn expanded_selection(
    nodes: &[Arc<NodeRecord>],
    ids: &[String],
    dialect: &dyn CanvasDialect,
    work: &mut WorkControl<'_>,
) -> Result<BTreeSet<String>> {
    if ids.len() > nodes.len() {
        return Err(CanvasError::Invalid(
            "movement selection exceeds board size".into(),
        ));
    }
    work.charge(nodes.len() + ids.len())?;
    let lookup: HashMap<_, _> = nodes.iter().map(|node| (node.id(), node)).collect();
    let mut selected = BTreeSet::new();
    let mut groups = vec![];
    for id in ids {
        if !selected.insert(id.clone()) {
            return Err(CanvasError::Invalid("duplicate movement identity".into()));
        }
        let node = lookup
            .get(id.as_str())
            .ok_or_else(|| CanvasError::NotFound(id.clone()))?;
        let policy = dialect.node_policy(node);
        if !policy.movable {
            return Err(CanvasError::NotEditable(id.clone()));
        }
        selected.insert(id.clone());
        if policy.group == GroupBehavior::Spatial {
            groups.push(node.bounds());
        }
    }
    for node in nodes {
        if selected.contains(node.id()) {
            continue;
        }
        for group in &groups {
            work.charge(1)?;
            if group.encloses(node.bounds()) && dialect.node_policy(node).movable {
                selected.insert(node.id().into());
                break;
            }
        }
    }
    Ok(selected)
}
fn put_rect(fields: &mut RawMap, rect: WorldRect) -> Result<()> {
    for (key, value) in [
        ("x", rect.x),
        ("y", rect.y),
        ("width", rect.width),
        ("height", rect.height),
    ] {
        fields.insert(key.into(), raw(&value)?);
    }
    Ok(())
}
fn with_rect(node: &NodeRecord, rect: WorldRect, limits: &CanvasLimits) -> Result<NodeRecord> {
    let mut fields = node.fields().clone();
    put_rect(&mut fields, rect)?;
    node.with_fields(fields, limits)
}
fn fields_equal(a: &RawMap, b: &RawMap) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|(key, value)| b.get(key).is_some_and(|other| other.get() == value.get()))
}
fn apply_fields(
    target: &mut RawMap,
    updates: Vec<FieldUpdate>,
    reserved: &[&str],
    limits: &CanvasLimits,
    work: &mut WorkControl<'_>,
) -> Result<()> {
    if updates.len() > limits.max_commands {
        return Err(CanvasError::Limit("field updates"));
    }
    let mut paths = HashSet::new();
    for update in updates {
        work.charge(update.path.len())?;
        FieldUpdate::validate_path(&update.path, limits)?;
        if !paths.insert(update.path.clone()) || reserved.contains(&update.path[0].as_str()) {
            return Err(CanvasError::Invalid(
                "duplicate or reserved field update".into(),
            ));
        }
        put_field(target, &update.path, update.value)?;
    }
    Ok(())
}
fn put_field(target: &mut RawMap, path: &[String], value: Option<Box<RawValue>>) -> Result<()> {
    let key = &path[0];
    if path.len() == 1 {
        if let Some(value) = value {
            target.insert(key.clone(), value);
        } else {
            target.remove(key);
        }
        return Ok(());
    }
    let mut child: RawMap = match target.get(key) {
        Some(raw) => serde_json::from_str(raw.get())?,
        None => BTreeMap::new(),
    };
    put_field(&mut child, &path[1..], value)?;
    target.insert(key.clone(), raw(&child)?);
    Ok(())
}
