//! Reusable native Markdown source draft for a host-owned dialog or editor pane.
use crate::CanvasViewLabels;
use craft_canvas_model::{
    CanvasCommand, CanvasError, CanvasSession, CanvasTransaction, ContentIdentity, NodeKind, Result,
};
use egui::{Id, Key, Ui};
/// A native source draft at one exact expected node/session, separate from canonical content.
pub struct TextCardEditor {
    base: ContentIdentity,
    node_id: String,
    expected: String,
    text: String,
    composing: bool,
    focus: bool,
    finished: bool,
}
/// One explicit native source-editor outcome; Cancel never emits a mutation.
#[derive(Default)]
pub struct TextEditorOutput {
    /// Exact-base edit requested by Save/Cmd-Enter; the host validates and acknowledges it.
    pub transaction: Option<CanvasTransaction>,
    /// The author explicitly cancelled this draft.
    pub cancelled: bool,
}
impl TextCardEditor {
    /// Capture the complete expected Markdown source from an exact text card.
    pub fn new(session: &CanvasSession, node_id: &str) -> Result<Self> {
        let node = session
            .document()
            .node(node_id)
            .ok_or_else(|| CanvasError::NotFound(node_id.into()))?;
        if !matches!(node.kind(), NodeKind::Text) {
            return Err(CanvasError::NotEditable(node_id.into()));
        }
        Ok(Self {
            base: session.identity(),
            node_id: node_id.into(),
            expected: node.body().into(),
            text: node.body().into(),
            composing: false,
            focus: true,
            finished: false,
        })
    }
    /// Current uncommitted source for host recovery/checkpointing.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Edit the uncommitted draft; no canonical write or authority is implied.
    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }
    /// Captured exact base for host conflict/recovery messaging.
    pub fn base(&self) -> ContentIdentity {
        self.base
    }
    /// Prepare a data-only request while retaining this draft on host validation/save failure.
    pub fn transaction(&self) -> CanvasTransaction {
        CanvasTransaction::new(
            self.base,
            vec![CanvasCommand::EditText {
                node_id: self.node_id.clone(),
                expected: self.expected.clone(),
                text: self.text.clone(),
            }],
        )
    }
    /// Show source and deliberate actions inside the host's common native modal/pane grid.
    /// Composition events do not become Save/Cancel shortcuts. Losing OS focus keeps the draft.
    pub fn show(&mut self, ui: &mut Ui, labels: &CanvasViewLabels) -> TextEditorOutput {
        if self.finished {
            return TextEditorOutput::default();
        }
        let ime_frame = ui.input(|input| {
            input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(_)))
        });
        ui.input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                        self.composing = !text.is_empty()
                    }
                    egui::Event::Ime(egui::ImeEvent::Commit(_)) => self.composing = false,
                    _ => {}
                }
            }
        });
        ui.label(&labels.text_source);
        let response = ui.add_sized(
            [ui.available_width(), 180.0],
            egui::TextEdit::multiline(&mut self.text)
                .id(Id::new(("craft-canvas-text", self.node_id.as_str())))
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(12, 8)),
        );
        if self.focus {
            response.request_focus();
            self.focus = false;
        }
        let save_key = !self.composing
            && !ime_frame
            && response.has_focus()
            && ui.input(|input| input.modifiers.command && input.key_pressed(Key::Enter));
        let cancel_key = !self.composing
            && !ime_frame
            && response.has_focus()
            && ui.input(|input| input.key_pressed(Key::Escape));
        let mut save = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            cancel = ui
                .add_sized([100.0, 40.0], egui::Button::new(&labels.cancel))
                .clicked();
            save = ui
                .add_enabled_ui(!self.composing && !ime_frame, |ui| {
                    ui.add_sized([100.0, 40.0], egui::Button::new(&labels.save))
                })
                .inner
                .clicked();
        });
        if cancel || cancel_key {
            self.finished = true;
            return TextEditorOutput {
                transaction: None,
                cancelled: true,
            };
        }
        if save || save_key {
            return TextEditorOutput {
                transaction: Some(self.transaction()),
                cancelled: false,
            };
        }
        TextEditorOutput::default()
    }
}
