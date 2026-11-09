//! Pure connection decisions shared by preview and final transaction validation.
use crate::{
    CanvasDialect, CanvasDocument, ConnectorEndpointKind, EdgeRecord, ExtensionStatus, PortBinding,
    PortEndpoint, PortKey, PortKind, ReconnectPolicy,
};

/// Direction compatibility for named and legacy connection endpoints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectionMode {
    /// A normalized connection runs from a source port to a target port.
    #[default]
    Strict,
    /// Matching port kinds are allowed, except a handle connected to itself.
    Loose,
}

/// The authoring operation whose policy is being checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionOperation {
    /// Create an additional edge; parallel edges are not implicitly deduplicated.
    Create {
        /// Planned caller-allocated identity, also present in the final candidate.
        /// Host capacity checks can exclude this edge consistently in preview and prepare.
        edge_id: String,
    },
    /// Change one endpoint while retaining the other exact binding.
    Reconnect {
        /// Stable identity of the original edge.
        edge_id: String,
        /// Endpoint deliberately being changed.
        endpoint: ConnectorEndpointKind,
    },
    /// Atomically change both endpoints of an existing stable edge.
    ReconnectBoth {
        /// Stable identity of the original edge.
        edge_id: String,
    },
}

/// A normalized connection proposal, never a prepared mutation or save authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionRequest {
    /// Canonical source binding after the proposed operation.
    pub from: PortEndpoint,
    /// Canonical destination binding after the proposed operation.
    pub to: PortEndpoint,
    /// Actual initiating endpoint, so start/end availability is independent of direction.
    /// A single-end reconnect starts at its unchanged opposite end; a two-end
    /// reconnect uses `From`, matching final transaction validation.
    pub start: ConnectorEndpointKind,
    /// Create or reconnect policy, including the original edge when applicable.
    pub operation: ConnectionOperation,
}

/// Actionable refusal from pure connection validation; no document is changed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConnectionError {
    /// An endpoint's node is absent from the candidate document.
    #[error("Connection node was not found: {0}")]
    MissingNode(String),
    /// A stable named port is absent; no fallback port is chosen.
    #[error("Connection port {port:?} was not found on node {node_id}")]
    MissingPort {
        /// Candidate node identity.
        node_id: String,
        /// Exact requested node-local port key.
        port: PortKey,
    },
    /// The required extension has unsupported semantics.
    #[error("Cannot interpret the preserved port extension on record {0}")]
    OpaqueExtension(String),
    /// The required version-one extension is malformed.
    #[error("Repair the malformed port extension on record {0} before connecting")]
    MalformedExtension(String),
    /// The host excludes this node from connection authoring.
    #[error("The node does not permit connection authoring: {0}")]
    NodeUnavailable(String),
    /// A named port is present but globally unavailable for connection authoring.
    #[error("The connection port is unavailable: {0:?}")]
    PortUnavailable(PortEndpoint),
    /// The initiating port does not permit starting a connection.
    #[error("This port cannot start a connection: {0:?}")]
    StartUnavailable(PortEndpoint),
    /// The receiving port does not permit completing a connection.
    #[error("This port cannot receive a connection: {0:?}")]
    EndUnavailable(PortEndpoint),
    /// Strict mode requires normalized source-to-target direction.
    #[error("Strict connections must run from a source port to a target port")]
    DirectionMismatch,
    /// Loose mode cannot connect the same node and textual handle identity to itself.
    #[error("A loose connection cannot connect a handle to itself")]
    SamePort,
    /// The original edge needed for reconnect authorization is absent.
    #[error("The original edge was not found for reconnection: {0}")]
    MissingEdge(String),
    /// The original edge policy excludes the deliberately changed endpoint.
    #[error("Edge {edge_id} does not permit reconnecting its {endpoint:?} endpoint")]
    ReconnectDisabled {
        /// Original edge identity.
        edge_id: String,
        /// Endpoint refused by the original policy.
        endpoint: ConnectorEndpointKind,
    },
    /// An inherited edge policy needs a concrete host default.
    #[error("The host reconnect default must be none, both, from or to, not inherit")]
    InvalidReconnectDefault,
    /// A single-end reconnect also changed its supposedly stationary endpoint.
    #[error("The stationary endpoint of edge {0} changed; prepare the operation again")]
    StationaryEndpointChanged(String),
    /// The supplied initiator contradicts the reconnect operation's defined semantics.
    #[error("This reconnection must start at {expected:?}, not {actual:?}")]
    InvalidInitiator {
        /// Initiating endpoint required by the operation.
        expected: ConnectorEndpointKind,
        /// Contradictory endpoint supplied by the caller.
        actual: ConnectorEndpointKind,
    },
    /// The host's additional connection rule refused the complete request.
    #[error("Connection refused by host policy: {0}")]
    HostRejected(String),
}

/// Check a proposal without changing canonical data or granting Apply authority.
///
/// `from` and `to` are normalized edge roles. A target-start gesture sets `start`
/// to `To`; its source remains `from`. Reconnection reads policy from this exact
/// document and requires an unchanged stationary endpoint. Strict mode permits
/// opposite-kind handles sharing an ID on one node; loose mode rejects that same
/// textual handle identity, matching the pinned React Flow behavior.
/// Reconnect requests must name the unchanged opposite endpoint as `start`;
/// `ReconnectBoth` must use `From`. Contradictory initiators are refused before
/// availability checks so preview and final preparation use the same semantics.
pub fn validate_connection(
    document: &CanvasDocument,
    request: &ConnectionRequest,
    dialect: &dyn CanvasDialect,
) -> std::result::Result<(), ConnectionError> {
    validate_connection_against(document, document, request, dialect)
}

/// Final preparation resolves endpoints in the candidate but authorizes changes
/// against original policy, so a simultaneous policy edit cannot grant itself access.
pub(crate) fn validate_connection_against(
    original: &CanvasDocument,
    candidate: &CanvasDocument,
    request: &ConnectionRequest,
    dialect: &dyn CanvasDialect,
) -> std::result::Result<(), ConnectionError> {
    validate_reconnect(original, request, dialect)?;
    let from = resolve(candidate, &request.from, PortKind::Source, dialect)?;
    let to = resolve(candidate, &request.to, PortKind::Target, dialect)?;
    match dialect.connection_mode() {
        ConnectionMode::Strict if from.kind != PortKind::Source || to.kind != PortKind::Target => {
            return Err(ConnectionError::DirectionMismatch);
        }
        ConnectionMode::Loose if same_handle(&request.from, &request.to) => {
            return Err(ConnectionError::SamePort);
        }
        _ => {}
    }
    let (start, start_info, end, end_info) = match request.start {
        ConnectorEndpointKind::From => (&request.from, from, &request.to, to),
        ConnectorEndpointKind::To => (&request.to, to, &request.from, from),
    };
    if !start_info.start {
        return Err(ConnectionError::StartUnavailable(start.clone()));
    }
    if !end_info.end {
        return Err(ConnectionError::EndUnavailable(end.clone()));
    }
    dialect.validate_connection(candidate, request)
}

struct ResolvedPort {
    kind: PortKind,
    start: bool,
    end: bool,
}

fn resolve(
    document: &CanvasDocument,
    endpoint: &PortEndpoint,
    legacy_kind: PortKind,
    dialect: &dyn CanvasDialect,
) -> std::result::Result<ResolvedPort, ConnectionError> {
    let node = document
        .node(&endpoint.node_id)
        .ok_or_else(|| ConnectionError::MissingNode(endpoint.node_id.clone()))?;
    if !dialect.can_connect(node) {
        return Err(ConnectionError::NodeUnavailable(node.id().into()));
    }
    match &endpoint.binding {
        PortBinding::Legacy(_) => Ok(ResolvedPort {
            kind: legacy_kind,
            start: true,
            end: true,
        }),
        PortBinding::Named(key) => {
            require_extension(node.port_extension_status(), node.id())?;
            let port = node.port(key).ok_or_else(|| ConnectionError::MissingPort {
                node_id: node.id().into(),
                port: key.clone(),
            })?;
            if !port.connectable {
                return Err(ConnectionError::PortUnavailable(endpoint.clone()));
            }
            Ok(ResolvedPort {
                kind: port.key.kind,
                start: port.start,
                end: port.end,
            })
        }
    }
}

fn require_extension(
    status: ExtensionStatus,
    id: &str,
) -> std::result::Result<(), ConnectionError> {
    match status {
        ExtensionStatus::Opaque => Err(ConnectionError::OpaqueExtension(id.into())),
        ExtensionStatus::Malformed => Err(ConnectionError::MalformedExtension(id.into())),
        ExtensionStatus::Absent | ExtensionStatus::KnownV1 => Ok(()),
    }
}

fn same_handle(from: &PortEndpoint, to: &PortEndpoint) -> bool {
    from.node_id == to.node_id
        && match (&from.binding, &to.binding) {
            (PortBinding::Named(a), PortBinding::Named(b)) => a.id == b.id,
            (PortBinding::Legacy(a), PortBinding::Legacy(b)) => a == b,
            _ => false,
        }
}

fn validate_reconnect(
    original: &CanvasDocument,
    request: &ConnectionRequest,
    dialect: &dyn CanvasDialect,
) -> std::result::Result<(), ConnectionError> {
    let (id, changed) = match &request.operation {
        ConnectionOperation::Create { .. } => return Ok(()),
        ConnectionOperation::Reconnect { edge_id, endpoint } => (edge_id, Some(*endpoint)),
        ConnectionOperation::ReconnectBoth { edge_id } => (edge_id, None),
    };
    let expected_start = match changed {
        Some(ConnectorEndpointKind::From) => ConnectorEndpointKind::To,
        Some(ConnectorEndpointKind::To) | None => ConnectorEndpointKind::From,
    };
    if request.start != expected_start {
        return Err(ConnectionError::InvalidInitiator {
            expected: expected_start,
            actual: request.start,
        });
    }
    let edge = original
        .edge(id)
        .ok_or_else(|| ConnectionError::MissingEdge(id.clone()))?;
    require_extension(edge.port_extension_status(), edge.id())?;
    let policy = edge
        .reconnect_policy()
        .map_err(|_| ConnectionError::MalformedExtension(edge.id().into()))?;
    let policy = match policy {
        ReconnectPolicy::Inherit => match dialect.default_reconnect_policy() {
            ReconnectPolicy::Inherit => return Err(ConnectionError::InvalidReconnectDefault),
            concrete => concrete,
        },
        concrete => concrete,
    };
    for endpoint in [ConnectorEndpointKind::From, ConnectorEndpointKind::To] {
        if changed.is_none_or(|changed| changed == endpoint) {
            let allowed = match policy {
                ReconnectPolicy::Both => true,
                ReconnectPolicy::From => endpoint == ConnectorEndpointKind::From,
                ReconnectPolicy::To => endpoint == ConnectorEndpointKind::To,
                ReconnectPolicy::Inherit | ReconnectPolicy::None => false,
            };
            if !allowed {
                return Err(ConnectionError::ReconnectDisabled {
                    edge_id: id.clone(),
                    endpoint,
                });
            }
        } else {
            let expected = endpoint_value(edge, endpoint)?;
            let actual = match endpoint {
                ConnectorEndpointKind::From => &request.from,
                ConnectorEndpointKind::To => &request.to,
            };
            if expected != *actual {
                return Err(ConnectionError::StationaryEndpointChanged(id.clone()));
            }
        }
    }
    Ok(())
}

fn endpoint_value(
    edge: &EdgeRecord,
    endpoint: ConnectorEndpointKind,
) -> std::result::Result<PortEndpoint, ConnectionError> {
    edge.endpoint(endpoint)
        .map_err(|_| ConnectionError::MalformedExtension(edge.id().into()))
}
