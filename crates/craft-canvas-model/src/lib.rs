//! Bounded JSON Canvas data and exact-base native/headless transactions.
//! Resource strings are inert; persistence and authorization belong to the host.
mod commands;
mod connections;
mod connectors;
mod document;
mod geometry;
mod json_guard;
mod ports;
mod session;
mod smooth_step;

pub use commands::{
    CanvasCommand, CanvasTransaction, FieldUpdate, IdMapping, Selection, SelectionMap, WorkControl,
};
pub use connections::{
    ConnectionError, ConnectionMode, ConnectionOperation, ConnectionRequest, validate_connection,
};
pub use connectors::{
    ConnectorEnd, ConnectorEndpoint, ConnectorEndpointKind, CubicConnector, LabelTarget,
    distance_to_segment,
};
pub use document::{CanvasDocument, CanvasLimits, EdgeRecord, NodeKind, NodeRecord, Side};
pub use geometry::{Camera, WorldPoint, WorldRect};
pub use ports::{
    ExtensionStatus, PortBinding, PortDefinition, PortEndpoint, PortKey, PortKind, ReconnectPolicy,
};
pub use session::{CanvasSession, ContentIdentity, HistoryUsage, PreparedChange};
pub use smooth_step::SmoothStepConnector;

/// A recoverable validation or exact-base failure. No failure changes the session.
#[derive(Debug, thiserror::Error)]
pub enum CanvasError {
    /// A proposed connection violates endpoint or reconnect policy.
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    /// Input is not complete valid JSON.
    #[error("Invalid Canvas JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Input contains two decoded keys in the same object.
    #[error("Duplicate Canvas JSON key: {0}")]
    DuplicateKey(String),
    /// A caller-configured capacity or geometry bound was exhausted.
    #[error("Canvas limit exceeded: {0}")]
    Limit(&'static str),
    /// Known JSON Canvas data violates its structural contract.
    #[error("Invalid Canvas data: {0}")]
    Invalid(String),
    /// A command refers to an absent stable node or edge.
    #[error("Canvas record was not found: {0}")]
    NotFound(String),
    /// A command's exact base is stale or belongs to another session.
    #[error("The Canvas changed; prepare this operation again")]
    StaleBase,
    /// A known edit would alter a host-constrained or unsupported node.
    #[error("Canvas record is not editable in this operation: {0}")]
    NotEditable(String),
    /// No complete Undo or Redo operation is available.
    #[error("No Canvas history is available for this operation")]
    NoHistory,
    /// A new complete reversible operation cannot fit its retained history budget.
    #[error("The complete Canvas operation exceeds the history budget")]
    HistoryBudget,
    /// Cooperative preparation was cancelled before mutation.
    #[error("Canvas operation cancelled")]
    Cancelled,
    /// Charged preparation budget was exhausted before mutation.
    #[error("Canvas operation work budget exhausted")]
    WorkLimit,
    /// Session identity or revision capacity has been exhausted.
    #[error("Canvas session identity capacity exhausted")]
    IdentityExhausted,
}
/// Result of a bounded Canvas operation.
pub type Result<T> = std::result::Result<T, CanvasError>;

/// Host-supplied semantic geometry policy. It performs no I/O or mutation.
pub trait CanvasDialect {
    /// Determine which generic geometric edits may act on a preserved record.
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy;
    /// Whether this canonical element may participate in a new/reconnected edge.
    /// Hosts exclude drawing strokes or other source-specific nonconnectable elements.
    fn can_connect(&self, node: &NodeRecord) -> bool {
        !matches!(node.kind(), NodeKind::Unknown(_))
    }
    /// Direction policy used consistently by pure previews and final preparation.
    fn connection_mode(&self) -> ConnectionMode {
        ConnectionMode::Strict
    }
    /// Concrete policy for edges with an absent or inherited reconnect setting.
    /// Returning `Inherit` is invalid because there is no further policy authority.
    fn default_reconnect_policy(&self) -> ReconnectPolicy {
        ReconnectPolicy::Both
    }
    /// Additional pure host rule after generic endpoint and original-edge checks.
    /// This hook receives the complete candidate; success grants no Apply authority.
    /// A create operation supplies its planned edge ID so capacity checks can exclude
    /// that edge when it is already present in the final candidate.
    /// Reconnect initiators have already been checked against operation semantics;
    /// a caller cannot bypass availability by supplying a contradictory start end.
    fn validate_connection(
        &self,
        _document: &CanvasDocument,
        _request: &ConnectionRequest,
    ) -> std::result::Result<(), ConnectionError> {
        Ok(())
    }
    /// Validate application extensions and authorized resource relationships.
    /// The default only accepts the generic structural validation already performed.
    fn validate(&self, _document: &CanvasDocument) -> Result<()> {
        Ok(())
    }
}
/// Behavior of a group-like record under selection geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupBehavior {
    /// A normal independent card, drawing or unsupported envelope.
    #[default]
    Independent,
    /// Moving/duplicating includes geometrically enclosed editable cards once.
    Spatial,
    /// A presentation boundary; moving it does not implicitly move enclosed cards.
    Frame,
}
/// Pure constraints supplied by the application's source dialect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodePolicy {
    /// Whether an explicit geometry change is supported and permitted.
    pub movable: bool,
    /// Whether resize is supported and permitted.
    pub resizable: bool,
    /// Enclosure behavior, separate from visible node kind.
    pub group: GroupBehavior,
}
/// Standard JSON Canvas semantics without application extension interpretation.
#[derive(Default)]
pub struct StandardDialect;
impl CanvasDialect for StandardDialect {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        let known = !matches!(node.kind(), NodeKind::Unknown(_));
        NodePolicy {
            movable: known,
            resizable: known,
            group: if matches!(node.kind(), NodeKind::Group) {
                GroupBehavior::Spatial
            } else {
                GroupBehavior::Independent
            },
        }
    }
}
