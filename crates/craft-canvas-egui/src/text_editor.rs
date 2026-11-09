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
    inline_viewport: Option<egui::ViewportId>,
    inline_activation: Option<InlineActivation>,
    inline_cancel_requested: bool,
    initial_arrow_navigation: bool,
    inline_geometry: Option<(egui::Rect, egui::Rect)>,
}
#[derive(Clone, Copy)]
struct InlineActivation {
    from_focus: Option<Id>,
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
            inline_viewport: None,
            inline_activation: None,
            inline_cancel_requested: false,
            initial_arrow_navigation: false,
            inline_geometry: None,
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
    /// Arm inline activation during the owning viewport's create/open UI pass.
    /// Focus is granted by the next unblocked pre-input hook, never after painting
    /// an absent widget into an otherwise complete accessibility tree. Call once
    /// per explicit activation, not repeatedly during painting.
    ///
    /// # Errors
    /// Refuses to move this retained input into a different viewport.
    pub fn begin_inline(&mut self, ctx: &egui::Context) -> Result<()> {
        let viewport = ctx.viewport_id();
        if self.inline_viewport.is_some_and(|bound| bound != viewport) {
            return Err(CanvasError::Invalid(
                "inline editor belongs to another viewport".into(),
            ));
        }
        self.inline_viewport = Some(viewport);
        let focused = ctx.memory(|memory| memory.focused());
        if focused != Some(self.id) {
            self.inline_activation = Some(InlineActivation {
                from_focus: focused,
            });
            self.focus = true;
            ctx.request_repaint();
        }
        Ok(())
    }
    /// Route only this active inline editor's Escape before egui focus navigation.
    /// Call from the host's `eframe::App::raw_input_hook` (or before `Context::run`).
    /// `blocked` includes any modal, inactive/disabled editor, culled card or host
    /// input handoff. The host must reveal and mount the active input in this pass.
    /// An armed activation only transfers its captured prior focus; competing
    /// focus is never overridden. It does not manufacture accessibility nodes.
    /// Pending commands survive repeated hooks before UI mounting (eframe may defer
    /// painting), but expire on blocked/foreign input or a disabled/unfocused mount.
    /// Text, paste and IME events are neither removed nor re-injected. Unknown focus
    /// or another viewport leaves all events untouched; this is not multiwindow routing.
    pub fn prepare_inline_input(
        &mut self,
        ctx: &egui::Context,
        input: &mut egui::RawInput,
        blocked: bool,
    ) {
        self.initial_arrow_navigation = false;
        if blocked || !input.focused || self.inline_viewport != Some(input.viewport_id) {
            self.inline_cancel_requested = false;
            return;
        }
        if let Some(activation) = self.inline_activation.take() {
            ctx.memory_mut(|memory| {
                if memory.focused() == activation.from_focus && !memory.has_focus(self.id) {
                    // Before begin_pass, an unmounted widget is subject to egui's
                    // ordinary focus cleanup before its accessibility update.
                    memory.request_focus(self.id);
                }
            });
        }
        if !ctx.memory(|memory| memory.has_focus(self.id)) {
            self.inline_cancel_requested = false;
            return;
        }
        let composing = self.composing
            || input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(_)));
        input.events.retain(|event| {
            if let egui::Event::Key {
                key: Key::Escape,
                pressed: true,
                repeat,
                modifiers,
                ..
            } = event
                && modifiers.is_none()
            {
                if !composing && !*repeat {
                    self.inline_cancel_requested = true;
                }
                false
            } else {
                true
            }
        });
        // egui installs TextEdit's key filter only after its first focused pass.
        // Detect only a batch whose pending focus direction can belong to this editor.
        self.initial_arrow_navigation = self.focus
            && input.events.iter().any(|event| matches!(event,
                egui::Event::Key { key: Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight,
                    pressed: true, modifiers, .. } if modifiers.is_none()))
            && input.events.iter().all(|event| match event {
                egui::Event::Key { key: Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight,
                    modifiers, .. } => modifiers.is_none(),
                egui::Event::Text(_) | egui::Event::PointerMoved(_) | egui::Event::MouseMoved(_) => true,
                _ => false,
            });
    }
    /// Mount the real multiline native input inside the card's reserved clipped body.
    /// Local selection, clipboard and Undo remain TextEdit-owned at the actual card zoom.
    /// IME composition and losing OS focus never become Done/Cancel or a board shortcut.
    pub fn show_inline(&mut self, ui: &mut Ui, rect: egui::Rect, zoom: f64) -> InlineTextOutput {
        if self
            .inline_viewport
            .is_some_and(|bound| bound != ui.ctx().viewport_id())
        {
            return InlineTextOutput::default();
        }
        let visible = rect.intersect(ui.clip_rect());
        if !visible.is_positive() {
            return InlineTextOutput::default();
        }
        let geometry_changed = self.inline_geometry != Some((rect, visible));
        let previous_clip_offset = self
            .inline_geometry
            .map_or(egui::Vec2::ZERO, |(body, clip)| clip.min - body.min);
        let clip_offset_delta = (visible.min - rect.min) - previous_clip_offset;
        self.inline_geometry = Some((rect, visible));
        let enabled = ui.is_enabled();
        let initial_focus =
            enabled && self.inline_activation.is_none() && std::mem::take(&mut self.focus);
        let immediate_focus = initial_focus && self.inline_viewport.is_none();
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let ime_frame = enabled
            && ui.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Ime(_)))
            });
        if enabled && ui.memory(|memory| memory.has_focus(self.id)) {
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
        }
        let composing =
            self.composing || (ime_frame && ui.memory(|memory| memory.has_focus(self.id)));
        if immediate_focus && !composing && !ime_frame && ui.input(|input| input.focused) {
            // Legacy immediate mounting acquires focus only where the actual
            // widget is about to participate in this pass's accessibility tree.
            // First-frame IME requires begin_inline plus prepare_inline_input.
            self.inline_viewport = Some(ui.ctx().viewport_id());
            ui.memory_mut(|memory| {
                if !memory.has_focus(self.id) {
                    memory.request_focus(self.id);
                }
            });
        }
        let owns_focus = enabled
            && ui.input(|input| input.focused)
            && ui.memory(|memory| memory.has_focus(self.id));
        if composing && owns_focus {
            // The OS IME owns these keys; TextEdit must not turn them into blur/newline actions.
            ui.input_mut(|input| {
                input.consume_key(egui::Modifiers::COMMAND, Key::Enter);
            });
        }
        let cancel_key =
            std::mem::take(&mut self.inline_cancel_requested) && owns_focus && !composing;
        let done_key = !cancel_key
            && !composing
            && owns_focus
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, Key::Enter));
        // A valid Cancel freezes this exact draft before same-batch text/paste can run.
        // The host's disabled state takes over once it receives the retained intent.
        let interactive = enabled && !cancel_key;
        let font = egui::FontId::monospace((13.0 * zoom as f32).clamp(8.0, 42.0));
        let margin = egui::Margin::same((6.0 * zoom as f32).clamp(3.0, 18.0) as i8);
        let response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt(("inline-card-viewport", self.id))
                    .max_rect(visible),
                |ui| {
                    ui.set_clip_rect(visible);
                    let salt = ("inline-card-scroll", self.id);
                    let mut scroll = egui::ScrollArea::both()
                        .id_salt(salt)
                        .max_width(visible.width())
                        .max_height(visible.height())
                        .min_scrolled_width(0.0)
                        .min_scrolled_height(0.0)
                        .auto_shrink([false, false])
                        .animated(false);
                    if clip_offset_delta != egui::Vec2::ZERO {
                        let offset =
                            egui::scroll_area::State::load(ui.ctx(), ui.make_persistent_id(salt))
                                .map_or(egui::Vec2::ZERO, |state| state.offset);
                        scroll = scroll
                            .scroll_offset((offset + clip_offset_delta).max(egui::Vec2::ZERO));
                    }
                    scroll
                        .show(ui, |ui| {
                            let output = egui::TextEdit::multiline(&mut self.text)
                                .id(self.id)
                                .interactive(interactive)
                                .event_filter(egui::EventFilter {
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    escape: true,
                                    tab: false,
                                })
                                .font(font.clone())
                                .desired_width(rect.width())
                                .desired_rows(1)
                                .frame(egui::Frame::new().inner_margin(margin))
                                .margin(margin)
                                // Keep full-card wrapping even when only a fragment is visible.
                                .min_size(rect.size())
                                .show(ui);
                            if geometry_changed
                                && output.response.has_focus()
                                && let Some(cursor) = output.cursor_range
                            {
                                let row_height = ui.fonts_mut(|fonts| fonts.row_height(&font));
                                let caret = egui::text_selection::text_cursor_state::cursor_rect(
                                    &output.galley,
                                    &cursor.primary,
                                    row_height,
                                )
                                .translate(
                                    output.galley_pos.to_vec2()
                                        - egui::vec2(output.galley.rect.left(), 0.0),
                                );
                                ui.scroll_to_rect(caret, None);
                            }
                            output.response
                        })
                        .inner
                },
            )
            .inner;
        if immediate_focus
            && !cancel_key
            && !composing
            && ui.input(|input| input.focused && input.pointer.primary_pressed())
            && ui.memory(|memory| memory.focused().is_none())
        {
            // An immediate mount may still share the creating toolbar click.
            // Never renew an existing focus or reset an active native composition.
            response.request_focus();
        }
        if std::mem::take(&mut self.initial_arrow_navigation) && enabled && response.has_focus() {
            ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        }
        let deliberate_blur = interactive
            && !immediate_focus
            && !composing
            && response.lost_focus()
            && ui.input(|input| {
                input.focused
                    && (input.pointer.primary_pressed()
                        || input.pointer.button_clicked(egui::PointerButton::Primary))
            });
        InlineTextOutput {
            changed: response.changed(),
            done_requested: done_key || deliberate_blur,
            cancelled: cancel_key,
            composing,
            input: crate::NodeBodyResponse {
                pointer_captured: ui.rect_contains_pointer(rect),
                keyboard_captured: enabled && (response.has_focus() || composing || cancel_key),
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
