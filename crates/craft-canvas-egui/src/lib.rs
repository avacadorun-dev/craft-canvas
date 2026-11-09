//! Native egui projection of one canonical Canvas session. No filesystem or clipboard I/O.
mod connection;
mod port_geometry;
mod text_editor;
mod view;
pub use connection::{
    CanvasInputOwnership, ConnectionCancelReason, ConnectionEndOutcome, ConnectionEvent,
    ConnectionGestureInfo, ConnectionLabels, ConnectionOptions, ConnectionPreview,
    ConnectionPreviewState, ConnectionSubmissionState,
};
pub use port_geometry::{PortAnchor, VisiblePortGeometry, VisibleReconnectHandle};
pub use text_editor::{InlineTextOutput, TextCardEditor, TextEditorOutput};
pub use view::{
    CanvasIntent, CanvasView, CanvasViewConfig, CanvasViewHost, CanvasViewLabels, CanvasViewOutput,
    CanvasViewStyle, NodeBodyResponse, PlainTextBody, VisibleEdgeGeometry, VisibleNodeGeometry,
};
