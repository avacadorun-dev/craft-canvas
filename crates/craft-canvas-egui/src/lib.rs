//! Native egui projection of one canonical Canvas session. No filesystem or clipboard I/O.
mod text_editor;
mod view;
pub use text_editor::{InlineTextOutput, TextCardEditor, TextEditorOutput};
pub use view::{
    CanvasIntent, CanvasView, CanvasViewConfig, CanvasViewHost, CanvasViewLabels, CanvasViewOutput,
    CanvasViewStyle, NodeBodyResponse, PlainTextBody, VisibleEdgeGeometry, VisibleNodeGeometry,
};
