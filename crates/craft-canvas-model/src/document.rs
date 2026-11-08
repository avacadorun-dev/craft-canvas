//! Raw record envelopes with a validated generic index.
use crate::{CanvasError, ConnectorEnd, Result, WorldPoint, WorldRect, json_guard::check_json};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};
use serde_json::value::RawValue;
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    sync::Arc,
};

pub(crate) type RawMap = BTreeMap<String, Box<RawValue>>;
/// Caller-configurable source-compatible capacity limits.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasLimits {
    /// Maximum original or generated snapshot bytes.
    pub max_bytes: usize,
    /// Maximum node count.
    pub max_nodes: usize,
    /// Maximum edge count.
    pub max_edges: usize,
    /// Maximum structural JSON depth.
    pub max_depth: usize,
    /// Maximum ID bytes, with IDs retained exactly rather than trimmed.
    pub max_id_bytes: usize,
    /// Maximum Unicode scalar count in a text card.
    pub max_text_chars: usize,
    /// Maximum commands in an externally supplied atomic transaction.
    pub max_commands: usize,
    /// Maximum magnitude of a node's x/y coordinate.
    pub max_coordinate: f64,
    /// Minimum node width and height.
    pub min_dimension: f64,
    /// Maximum node width and height.
    pub max_dimension: f64,
    /// Require mathematically integral stored coordinates/dimensions.
    pub integral_geometry: bool,
    /// Maximum charged preparation work units before explicit refusal.
    pub max_work_units: usize,
    /// Maximum complete session Undo steps.
    pub max_history_steps: usize,
    /// Maximum past/future raw snapshot bytes. The current source has the separate
    /// max_bytes bound; parsed indexes and mapping overhead are reported separately.
    pub max_history_bytes: usize,
}
impl Default for CanvasLimits {
    fn default() -> Self {
        Self {
            max_bytes: 12 * 1024 * 1024,
            max_nodes: 5_000,
            max_edges: 10_000,
            max_depth: 64,
            max_id_bytes: 200,
            max_text_chars: 100_000,
            max_commands: 500,
            max_coordinate: 10_000_000.0,
            min_dimension: 40.0,
            max_dimension: 10_000.0,
            integral_geometry: true,
            max_work_units: 8_000_000,
            max_history_steps: 200,
            max_history_bytes: 32 * 1024 * 1024,
        }
    }
}
impl CanvasLimits {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.max_bytes == 0
            || self.max_nodes == 0
            || self.max_edges == 0
            || self.max_depth == 0
            || self.max_depth > 96
            || self.max_id_bytes == 0
            || self.max_commands == 0
            || self.max_history_steps == 0
            || !self.max_coordinate.is_finite()
            || self.max_coordinate <= 0.0
            || !self.min_dimension.is_finite()
            || self.min_dimension <= 0.0
            || !self.max_dimension.is_finite()
            || self.max_dimension < self.min_dimension
        {
            return Err(CanvasError::Invalid("invalid Canvas limits".into()));
        }
        Ok(())
    }
}
/// Standard node kinds; future kinds are retained without fabricated editing support.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// Markdown text card.
    Text,
    /// Inert file/resource reference.
    File,
    /// Inert URL card.
    Link,
    /// Spatial group; extension semantics remain host-owned.
    Group,
    /// Preserved future kind.
    Unknown(String),
}
/// A validated preserved node. Fields cannot be forged by deserialization.
#[derive(Debug)]
pub struct NodeRecord {
    id: String,
    kind: NodeKind,
    bounds: WorldRect,
    body: String,
    raw: Box<RawValue>,
    fields: RawMap,
}
impl NodeRecord {
    /// Construct a standard text card with a caller-allocated stable ID.
    pub fn text(
        id: String,
        bounds: WorldRect,
        text: String,
        limits: &CanvasLimits,
    ) -> Result<Self> {
        let mut fields = RawMap::new();
        for (key, value) in [
            ("id", raw(&id)?),
            ("type", raw(&"text")?),
            ("x", raw(&bounds.x)?),
            ("y", raw(&bounds.y)?),
            ("width", raw(&bounds.width)?),
            ("height", raw(&bounds.height)?),
            ("text", raw(&text)?),
        ] {
            fields.insert(key.into(), value);
        }
        Self::parse(&serde_json::to_string(&fields)?, limits)
    }
    /// Parse a standalone node without assigning project/resource authority.
    pub fn parse(raw: &str, limits: &CanvasLimits) -> Result<Self> {
        limits.validate()?;
        check_json(raw.as_bytes(), limits)?;
        Self::from_raw(serde_json::from_str(raw)?, limits)
    }
    pub(crate) fn from_raw(raw: Box<RawValue>, limits: &CanvasLimits) -> Result<Self> {
        let fields: RawMap = serde_json::from_str(raw.get())?;
        let id = required_string(&fields, "id")?;
        valid_id(&id, limits)?;
        let name = required_string(&fields, "type")?;
        let kind = match name.as_str() {
            "text" => NodeKind::Text,
            "file" => NodeKind::File,
            "link" => NodeKind::Link,
            "group" => NodeKind::Group,
            _ if !name.trim().is_empty() && name.len() <= limits.max_id_bytes => {
                NodeKind::Unknown(name)
            }
            _ => return Err(CanvasError::Invalid("node type missing".into())),
        };
        let bounds = WorldRect {
            x: number(&fields, "x")?,
            y: number(&fields, "y")?,
            width: number(&fields, "width")?,
            height: number(&fields, "height")?,
        }
        .validate(limits)?;
        let body = match kind {
            NodeKind::Text => required_string(&fields, "text")?,
            NodeKind::File => required_string(&fields, "file")?,
            NodeKind::Link => required_string(&fields, "url")?,
            NodeKind::Group => optional_string(&fields, "label")?.unwrap_or_default(),
            NodeKind::Unknown(_) => String::new(),
        };
        if matches!(kind, NodeKind::Text) && body.chars().count() > limits.max_text_chars {
            return Err(CanvasError::Limit("text card characters"));
        }
        let oversized_reference = match kind {
            NodeKind::File => body.chars().count() > 1_024,
            NodeKind::Link => body.len() > 2_048,
            _ => false,
        };
        if matches!(kind, NodeKind::File | NodeKind::Link)
            && (body.trim().is_empty() || oversized_reference)
        {
            return Err(CanvasError::Invalid(
                "empty or oversized file/URL reference".into(),
            ));
        }
        Ok(Self {
            id,
            kind,
            bounds,
            body,
            raw,
            fields,
        })
    }
    /// Exact stable identity.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Parsed node kind.
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }
    /// Validated world geometry.
    pub fn bounds(&self) -> WorldRect {
        self.bounds
    }
    /// Text, file, URL or group label for a generic inert presentation.
    pub fn body(&self) -> &str {
        &self.body
    }
    /// Unmodified or deliberately merged record JSON.
    pub fn raw_json(&self) -> &str {
        self.raw.get()
    }
    /// Borrow a preserved field without interpreting its authority.
    pub fn field(&self, key: &str) -> Option<&RawValue> {
        self.fields.get(key).map(Box::as_ref)
    }
    pub(crate) fn with_fields(&self, fields: RawMap, limits: &CanvasLimits) -> Result<Self> {
        Self::from_raw(
            RawValue::from_string(serde_json::to_string(&fields)?)?,
            limits,
        )
    }
    pub(crate) fn fields(&self) -> &RawMap {
        &self.fields
    }
    pub(crate) fn payload_bytes(&self) -> usize {
        self.raw.get().len()
            + self.id.len()
            + self.body.len()
            + self
                .fields
                .iter()
                .map(|(key, value)| key.len() + value.get().len())
                .sum::<usize>()
    }
}
/// A standard connector side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Top midpoint.
    Top,
    /// Right midpoint.
    Right,
    /// Bottom midpoint.
    Bottom,
    /// Left midpoint.
    Left,
}
impl Side {
    /// Four canonical handle sides in stable source order.
    pub const ALL: [Self; 4] = [Self::Top, Self::Right, Self::Bottom, Self::Left];
    /// Stable JSON Canvas spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
    /// Side midpoint in world coordinates.
    pub fn point(self, rect: WorldRect) -> WorldPoint {
        match self {
            Self::Top => WorldPoint {
                x: rect.x + rect.width / 2.0,
                y: rect.y,
            },
            Self::Right => WorldPoint {
                x: rect.x + rect.width,
                y: rect.y + rect.height / 2.0,
            },
            Self::Bottom => WorldPoint {
                x: rect.x + rect.width / 2.0,
                y: rect.y + rect.height,
            },
            Self::Left => WorldPoint {
                x: rect.x,
                y: rect.y + rect.height / 2.0,
            },
        }
    }
    fn parse(value: Option<String>) -> Result<Option<Self>> {
        match value.as_deref() {
            None | Some("") => Ok(None),
            Some("top") => Ok(Some(Self::Top)),
            Some("right") => Ok(Some(Self::Right)),
            Some("bottom") => Ok(Some(Self::Bottom)),
            Some("left") => Ok(Some(Self::Left)),
            _ => Err(CanvasError::Invalid("invalid connector side".into())),
        }
    }
}
/// A validated connector preserving every opaque field.
#[derive(Debug)]
pub struct EdgeRecord {
    id: String,
    from: String,
    to: String,
    from_side: Option<Side>,
    to_side: Option<Side>,
    from_end: ConnectorEnd,
    to_end: ConnectorEnd,
    label: String,
    raw: Box<RawValue>,
    fields: RawMap,
}
impl EdgeRecord {
    /// Construct a standard connector with explicit caller-owned identity and endpoints.
    pub fn connect(
        id: String,
        from: String,
        from_side: Side,
        to: String,
        to_side: Side,
        limits: &CanvasLimits,
    ) -> Result<Self> {
        let mut fields = RawMap::new();
        for (key, value) in [
            ("id", raw(&id)?),
            ("fromNode", raw(&from)?),
            ("fromSide", raw(&from_side.as_str())?),
            ("toNode", raw(&to)?),
            ("toSide", raw(&to_side.as_str())?),
        ] {
            fields.insert(key.into(), value);
        }
        Self::parse(&serde_json::to_string(&fields)?, limits)
    }
    /// Parse a standalone connector; document validation additionally checks endpoints.
    pub fn parse(raw: &str, limits: &CanvasLimits) -> Result<Self> {
        limits.validate()?;
        check_json(raw.as_bytes(), limits)?;
        Self::from_raw(serde_json::from_str(raw)?, limits)
    }
    pub(crate) fn from_raw(raw: Box<RawValue>, limits: &CanvasLimits) -> Result<Self> {
        let fields: RawMap = serde_json::from_str(raw.get())?;
        let id = required_string(&fields, "id")?;
        valid_id(&id, limits)?;
        let from = required_string(&fields, "fromNode")?;
        valid_id(&from, limits)?;
        let to = required_string(&fields, "toNode")?;
        valid_id(&to, limits)?;
        let from_side = Side::parse(optional_string(&fields, "fromSide")?)?;
        let to_side = Side::parse(optional_string(&fields, "toSide")?)?;
        for key in ["fromEnd", "toEnd"] {
            if let Some(end) = optional_string(&fields, key)?
                && !["", "none", "arrow"].contains(&end.as_str())
            {
                return Err(CanvasError::Invalid("invalid connector end".into()));
            }
        }
        let from_end = if optional_string(&fields, "fromEnd")?.as_deref() == Some("arrow") {
            ConnectorEnd::Arrow
        } else {
            ConnectorEnd::None
        };
        let to_end = if optional_string(&fields, "toEnd")?.as_deref() == Some("none") {
            ConnectorEnd::None
        } else {
            ConnectorEnd::Arrow
        };
        let label = optional_string(&fields, "label")?.unwrap_or_default();
        if label.chars().count() > 500 {
            return Err(CanvasError::Limit("edge label characters"));
        }
        Ok(Self {
            id,
            from,
            to,
            from_side,
            to_side,
            from_end,
            to_end,
            label,
            raw,
            fields,
        })
    }
    /// Stable edge identity.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Source node identity.
    pub fn from_node(&self) -> &str {
        &self.from
    }
    /// Destination node identity.
    pub fn to_node(&self) -> &str {
        &self.to
    }
    /// Explicit source side, if present.
    pub fn from_side(&self) -> Option<Side> {
        self.from_side
    }
    /// Explicit destination side, if present.
    pub fn to_side(&self) -> Option<Side> {
        self.to_side
    }
    /// Source marker, with the JSON Canvas default of no arrow.
    pub fn from_end(&self) -> ConnectorEnd {
        self.from_end
    }
    /// Destination marker, with the JSON Canvas default of an arrow.
    pub fn to_end(&self) -> ConnectorEnd {
        self.to_end
    }
    /// Preserved visible label.
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Exact preserved record JSON.
    pub fn raw_json(&self) -> &str {
        self.raw.get()
    }
    /// Borrow a preserved field.
    pub fn field(&self, key: &str) -> Option<&RawValue> {
        self.fields.get(key).map(Box::as_ref)
    }
    pub(crate) fn fields(&self) -> &RawMap {
        &self.fields
    }
    pub(crate) fn with_fields(&self, fields: RawMap, limits: &CanvasLimits) -> Result<Self> {
        Self::from_raw(
            RawValue::from_string(serde_json::to_string(&fields)?)?,
            limits,
        )
    }
    pub(crate) fn payload_bytes(&self) -> usize {
        self.raw.get().len()
            + self.id.len()
            + self.from.len()
            + self.to.len()
            + self.label.len()
            + self
                .fields
                .iter()
                .map(|(key, value)| key.len() + value.get().len())
                .sum::<usize>()
    }
}
#[derive(Debug)]
pub(crate) struct DocumentData {
    pub raw: Arc<str>,
    pub top: Arc<RawMap>,
    pub nodes: Vec<Arc<NodeRecord>>,
    pub edges: Vec<Arc<EdgeRecord>>,
    node_index: HashMap<String, usize>,
    edge_index: HashMap<String, usize>,
}
/// Immutable canonical board and generic index; cloning shares preserved records.
#[derive(Clone, Debug)]
pub struct CanvasDocument {
    pub(crate) data: Arc<DocumentData>,
}
impl CanvasDocument {
    /// Parse a bounded complete board. No-edit output is byte-for-byte identical.
    pub fn parse(raw: &str, limits: &CanvasLimits) -> Result<Self> {
        limits.validate()?;
        check_json(raw.as_bytes(), limits)?;
        let mut top: RawMap = serde_json::from_str(raw)?;
        fn records(top: &mut RawMap, key: &str) -> Result<Vec<Box<RawValue>>> {
            match top.remove(key) {
                None => Ok(vec![]),
                Some(raw) if raw.get() == "null" => Ok(vec![]),
                Some(raw) => Ok(serde_json::from_str(raw.get())?),
            }
        }
        let nodes = records(&mut top, "nodes")?
            .into_iter()
            .map(|raw| NodeRecord::from_raw(raw, limits).map(Arc::new))
            .collect::<Result<Vec<_>>>()?;
        let edges = records(&mut top, "edges")?
            .into_iter()
            .map(|raw| EdgeRecord::from_raw(raw, limits).map(Arc::new))
            .collect::<Result<Vec<_>>>()?;
        Self::build(Arc::from(raw), Arc::new(top), nodes, edges, limits)
    }
    pub(crate) fn build(
        raw: Arc<str>,
        top: Arc<RawMap>,
        nodes: Vec<Arc<NodeRecord>>,
        edges: Vec<Arc<EdgeRecord>>,
        limits: &CanvasLimits,
    ) -> Result<Self> {
        if raw.len() > limits.max_bytes
            || nodes.len() > limits.max_nodes
            || edges.len() > limits.max_edges
        {
            return Err(CanvasError::Limit("snapshot/graph size"));
        }
        let mut node_index = HashMap::with_capacity(nodes.len());
        for (at, node) in nodes.iter().enumerate() {
            if node_index.insert(node.id.clone(), at).is_some() {
                return Err(CanvasError::Invalid("duplicate node ID".into()));
            }
        }
        let mut edge_index = HashMap::with_capacity(edges.len());
        for (at, edge) in edges.iter().enumerate() {
            if edge_index.insert(edge.id.clone(), at).is_some() {
                return Err(CanvasError::Invalid("duplicate edge ID".into()));
            }
            if !node_index.contains_key(&edge.from) || !node_index.contains_key(&edge.to) {
                return Err(CanvasError::Invalid("unknown connector endpoint".into()));
            }
        }
        Ok(Self {
            data: Arc::new(DocumentData {
                raw,
                top,
                nodes,
                edges,
                node_index,
                edge_index,
            }),
        })
    }
    pub(crate) fn from_parts(
        top: Arc<RawMap>,
        nodes: Vec<Arc<NodeRecord>>,
        edges: Vec<Arc<EdgeRecord>>,
        limits: &CanvasLimits,
    ) -> Result<Self> {
        let mut output = BoundedWriter {
            bytes: Vec::new(),
            limit: limits.max_bytes,
        };
        serde_json::to_writer(
            &mut output,
            &Output {
                top: &top,
                nodes: &nodes,
                edges: &edges,
            },
        )?;
        let raw = String::from_utf8(output.bytes)
            .map_err(|_| CanvasError::Invalid("generated JSON encoding".into()))?;
        check_json(raw.as_bytes(), limits)?;
        Self::build(Arc::from(raw), top, nodes, edges, limits)
    }
    /// Original bytes until a real edit; generated complete bytes afterwards.
    pub fn raw_json(&self) -> &str {
        &self.data.raw
    }
    /// Nodes in canonical paint/source order.
    pub fn nodes(&self) -> &[Arc<NodeRecord>] {
        &self.data.nodes
    }
    /// Edges in canonical source order.
    pub fn edges(&self) -> &[Arc<EdgeRecord>] {
        &self.data.edges
    }
    /// Resolve a node only in this board.
    pub fn node(&self, id: &str) -> Option<&NodeRecord> {
        self.data
            .node_index
            .get(id)
            .map(|at| self.data.nodes[*at].as_ref())
    }
    /// Resolve an edge only in this board.
    pub fn edge(&self, id: &str) -> Option<&EdgeRecord> {
        self.data
            .edge_index
            .get(id)
            .map(|at| self.data.edges[*at].as_ref())
    }
    /// Borrow a preserved non-node/edge top-level field.
    pub fn field(&self, key: &str) -> Option<&RawValue> {
        self.data.top.get(key).map(Box::as_ref)
    }
    /// Expand a movement selection using the same group/lock policy as transaction preparation.
    pub fn expand_move_selection(
        &self,
        ids: &[String],
        dialect: &dyn crate::CanvasDialect,
        work: &mut crate::WorkControl<'_>,
    ) -> Result<std::collections::BTreeSet<String>> {
        crate::commands::expanded_selection(self.nodes(), ids, dialect, work)
    }
    /// Combined node bounds, or None for an empty graph.
    pub fn bounds(&self) -> Option<WorldRect> {
        self.nodes()
            .iter()
            .map(|node| node.bounds())
            .reduce(WorldRect::union)
    }
    /// Topmost matching node, retaining canonical order.
    pub fn hit_node(&self, point: WorldPoint) -> Option<&NodeRecord> {
        self.nodes()
            .iter()
            .rev()
            .find(|node| node.bounds().contains(point))
            .map(Arc::as_ref)
    }
}
struct Output<'a> {
    top: &'a RawMap,
    nodes: &'a [Arc<NodeRecord>],
    edges: &'a [Arc<EdgeRecord>],
}
impl Serialize for Output<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.top.len() + 2))?;
        for (key, value) in self.top {
            map.serialize_entry(key, value)?;
        }
        map.serialize_entry("nodes", &Nodes(self.nodes))?;
        map.serialize_entry("edges", &Edges(self.edges))?;
        map.end()
    }
}
struct Nodes<'a>(&'a [Arc<NodeRecord>]);
impl Serialize for Nodes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for node in self.0 {
            seq.serialize_element(&node.raw)?;
        }
        seq.end()
    }
}
struct Edges<'a>(&'a [Arc<EdgeRecord>]);
impl Serialize for Edges<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for edge in self.0 {
            seq.serialize_element(&edge.raw)?;
        }
        seq.end()
    }
}
struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|len| len > self.limit)
        {
            return Err(std::io::Error::other("Canvas output byte limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn required_string(fields: &RawMap, key: &str) -> Result<String> {
    optional_string(fields, key)?.ok_or_else(|| CanvasError::Invalid(format!("missing {key}")))
}
pub(crate) fn optional_string(fields: &RawMap, key: &str) -> Result<Option<String>> {
    fields
        .get(key)
        .map(|raw| serde_json::from_str(raw.get()).map_err(Into::into))
        .transpose()
}
fn number(fields: &RawMap, key: &str) -> Result<f64> {
    serde_json::from_str(
        fields
            .get(key)
            .ok_or_else(|| CanvasError::Invalid(format!("missing {key}")))?
            .get(),
    )
    .map_err(Into::into)
}
pub(crate) fn valid_id(id: &str, limits: &CanvasLimits) -> Result<()> {
    if id.trim().is_empty() || id.trim().len() > limits.max_id_bytes {
        Err(CanvasError::Invalid("empty or oversized ID".into()))
    } else {
        Ok(())
    }
}
pub(crate) fn raw<T: Serialize>(value: &T) -> Result<Box<RawValue>> {
    Ok(RawValue::from_string(serde_json::to_string(value)?)?)
}
