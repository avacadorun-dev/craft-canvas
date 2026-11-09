//! Reusable native Markdown source draft for a host-owned dialog or editor pane.
use crate::CanvasViewLabels;
use craft_canvas_model::{
    CanvasCommand, CanvasError, CanvasSession, CanvasTransaction, ContentIdentity, NodeKind,
    NodeRecord, Result,
};
use egui::{Id, Key, Ui};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
static NEXT_EDITOR: AtomicU64 = AtomicU64::new(1);
/// A native source draft at one exact expected node/session, separate from canonical content.
pub struct TextCardEditor {
    base: ContentIdentity,
    node_id: String,
    expected: String,
    text: String,
    composing: bool,
    focus: bool,
    finished: bool,
    id: Id,
    submitted: Option<String>,
    insertion: Option<Arc<NodeRecord>>,
}
/// One explicit native source-editor outcome; Cancel never emits a mutation.
#[derive(Default)]
pub struct TextEditorOutput {
    /// Exact-base edit requested by Save/Cmd-Enter; the host validates and acknowledges it.
    pub transaction: Option<CanvasTransaction>,
    /// The author explicitly cancelled this draft.
    pub cancelled: bool,
}
/// Native in-card editor outcome. Canonical save/acknowledgement remains the host's job.
#[derive(Default)]
pub struct InlineTextOutput {
    /// Local native text buffer changed; it is not yet a canonical save acknowledgement.
    pub changed: bool,
    /// Author committed with Cmd/Ctrl-Enter or a deliberate same-window pointer blur.
    pub done_requested: bool,
    /// Escape requested discarding the unacknowledged local edit.
    pub cancelled: bool,
    /// A native composition is active or this frame carried an IME event.
    pub composing: bool,
    /// Native child controls own this frame's pointer/keyboard rather than a board gesture.
    pub input: crate::NodeBodyResponse,
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
        Self::from_node(session, node, None)
    }
    /// Own native input for an absent text card while the host persists its insertion.
    /// This only creates a presentation draft, never a second canonical session.
    pub fn pending_insertion(session: &CanvasSession, node: NodeRecord) -> Result<Self> {
        let node = Arc::new(NodeRecord::parse(node.raw_json(), session.limits())?);
        if session.document().node(node.id()).is_some() || !matches!(node.kind(), NodeKind::Text) {
            return Err(CanvasError::NotEditable(node.id().into()));
        }
        Self::from_node(session, &node, Some(Arc::clone(&node)))
    }
    fn from_node(
        session: &CanvasSession,
        node: &NodeRecord,
        insertion: Option<Arc<NodeRecord>>,
    ) -> Result<Self> {
        Ok(Self {
            base: session.identity(),
            node_id: node.id().into(),
            expected: node.body().into(),
            text: node.body().into(),
            composing: false,
            focus: true,
            finished: false,
            id: Id::new((
                "craft-canvas-text-editor",
                NEXT_EDITOR
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                        value.checked_add(1)
                    })
                    .map_err(|_| CanvasError::Invalid("native editor identity exhausted".into()))?,
            )),
            submitted: None,
            insertion,
        })
    }
    /// Whether this draft still awaits the host's exact insertion acknowledgement.
    pub fn awaits_insertion(&self) -> bool {
        self.insertion.is_some()
    }
    /// Bounded provisional presentation at its exact unchanged source session.
    /// The host may render this card while board interactions are frozen.
    pub fn pending_node(&self, session: &CanvasSession) -> Option<Arc<NodeRecord>> {
        (self.base == session.identity())
            .then(|| self.insertion.as_ref().map(Arc::clone))
            .flatten()
    }
    /// Promote the same native draft only after the exact inserted record is applied.
    /// Text, widget identity, selection, composition and local Undo remain untouched.
    pub fn acknowledge_insertion(&mut self, session: &CanvasSession) -> Result<()> {
        let pending = self
            .insertion
            .as_ref()
            .ok_or_else(|| CanvasError::Invalid("text card has no pending insertion".into()))?;
        if !self.base.same_session(session.identity())
            || self.base.revision().checked_add(1) != Some(session.identity().revision())
            || session
                .document()
                .node(pending.id())
                .map(NodeRecord::raw_json)
                != Some(pending.raw_json())
        {
            return Err(CanvasError::StaleBase);
        }
        self.base = session.identity();
        self.insertion = None;
        Ok(())
    }
    /// Stable selected card identity; never an ambient resource path.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    /// Whether local text differs from the last acknowledged canonical source.
    pub fn dirty(&self) -> bool {
        self.text != self.expected
    }
    /// Whether an exact submitted edit still awaits host persistence acknowledgement.
    pub fn has_submission(&self) -> bool {
        self.submitted.is_some()
    }
    /// Capture one submitted source while allowing later typing to remain in this same buffer.
    pub fn submit(&mut self) -> Result<CanvasTransaction> {
        if self.insertion.is_some() {
            return Err(CanvasError::Invalid(
                "text card insertion awaits acknowledgement".into(),
            ));
        }
        if self.submitted.is_some() {
            return Err(CanvasError::Invalid(
                "text edit already awaits acknowledgement".into(),
            ));
        }
        let transaction = self.transaction();
        self.submitted = Some(self.text.clone());
        Ok(transaction)
    }
    /// Undo only the local submission marker after the host refuses to start a write.
    /// This does not discard text or certify that an already-started write was rolled back.
    pub fn submission_not_started(&mut self) {
        self.submitted = None;
    }
    /// Rebase only after the host durably saved and applied this exact submitted source.
    /// Later typing and the native widget identity/selection/history remain unchanged.
    pub fn acknowledge(&mut self, session: &CanvasSession) -> Result<()> {
        let submitted = self
            .submitted
            .as_ref()
            .ok_or_else(|| CanvasError::Invalid("text edit has no submitted source".into()))?;
        if !self.base.same_session(session.identity())
            || session.identity().revision() < self.base.revision()
        {
            return Err(CanvasError::StaleBase);
        }
        let node = session
            .document()
            .node(&self.node_id)
            .ok_or_else(|| CanvasError::NotFound(self.node_id.clone()))?;
        if !matches!(node.kind(), NodeKind::Text) || node.body() != submitted {
            return Err(CanvasError::StaleBase);
        }
        self.expected = submitted.clone();
        self.base = session.identity();
        self.submitted = None;
        Ok(())
    }
    /// Mount the real multiline native input inside the card's reserved clipped body.
    /// Local selection, clipboard and Undo remain TextEdit-owned at the actual card zoom.
    /// IME composition and losing OS focus never become Done/Cancel or a board shortcut.
    pub fn show_inline(&mut self, ui: &mut Ui, rect: egui::Rect, zoom: f64) -> InlineTextOutput {
        let initial_focus = std::mem::take(&mut self.focus);
        if initial_focus {
            // A newly mounted card can already have typing in this frame's event queue.
            ui.memory_mut(|memory| memory.request_focus(self.id));
        }
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
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
        let composing = self.composing || ime_frame;
        let owns_focus = ui.memory(|memory| memory.has_focus(self.id));
        if composing && owns_focus {
            // The OS IME owns these keys; TextEdit must not turn them into blur/newline actions.
            ui.input_mut(|input| {
                input.consume_key(egui::Modifiers::COMMAND, Key::Enter);
                input.consume_key(egui::Modifiers::NONE, Key::Escape);
            });
        }
        let done_key = !composing
            && owns_focus
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, Key::Enter));
        let cancel_key = !composing
            && owns_focus
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape));
        let response = egui::ScrollArea::vertical()
            .id_salt(("inline-card-scroll", self.id))
            .max_height(rect.height())
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text)
                        .id(self.id)
                        .font(egui::FontId::monospace(
                            (13.0 * zoom as f32).clamp(8.0, 42.0),
                        ))
                        .desired_width(rect.width())
                        .desired_rows(1)
                        .frame(egui::Frame::new())
                        .min_size(egui::vec2(0.0, rect.height()))
                        .margin(egui::Margin::same(
                            (6.0 * zoom as f32).clamp(3.0, 18.0) as i8
                        )),
                )
            })
            .inner;
        if initial_focus {
            // The creating toolbar/board click must not take focus back from this card.
            response.request_focus();
        }
        let deliberate_blur = !initial_focus
            && !composing
            && response.lost_focus()
            && ui.input(|input| input.focused && input.pointer.primary_pressed());
        InlineTextOutput {
            changed: response.changed(),
            done_requested: done_key || deliberate_blur,
            cancelled: cancel_key,
            composing,
            input: crate::NodeBodyResponse {
                pointer_captured: ui.rect_contains_pointer(rect),
                keyboard_captured: response.has_focus() || composing,
            },
        }
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
                .id(self.id)
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
        if (save || save_key) && !self.awaits_insertion() {
            return TextEditorOutput {
                transaction: Some(self.transaction()),
                cancelled: false,
            };
        }
        TextEditorOutput::default()
    }
}
