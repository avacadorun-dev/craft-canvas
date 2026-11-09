//! A relative-origin native viewport emits guarded intents instead of saving another model.
use craft_canvas_model::{
    Camera, CanvasCommand, CanvasDialect, CanvasError, CanvasSession, CanvasTransaction,
    ConnectorEnd, ContentIdentity, GroupBehavior, NodeKind, NodeRecord, Selection, Side,
    SmoothStepConnector, WorkControl, WorldPoint, WorldRect, distance_to_segment,
};
use egui::{
    Align2, Color32, FontId, Id, Key, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
    Vec2, WidgetInfo, WidgetType,
};
use std::collections::BTreeSet;

/// Host-localized labels. Generic defaults are for examples, not an app locale catalog.
#[derive(Clone, Debug)]
pub struct CanvasViewLabels {
    /// Accessible viewport name.
    pub canvas: String,
    /// Hover hint for pointer/keyboard entry into an editable text card.
    pub edit_text_hint: String,
    /// Explicit save action.
    pub save: String,
    /// Explicit cancel action.
    pub cancel: String,
    /// Accessible text source label.
    pub text_source: String,
}
impl Default for CanvasViewLabels {
    fn default() -> Self {
        Self {
            canvas: "Canvas".into(),
            edit_text_hint: "Double-click or press Enter to edit".into(),
            save: "Save".into(),
            cancel: "Cancel".into(),
            text_source: "Markdown source".into(),
        }
    }
}
/// Coherent host theme values; no separate generic palette system is introduced.
#[derive(Clone, Copy, Debug)]
pub struct CanvasViewStyle {
    /// View background.
    pub background: Color32,
    /// Card surface.
    pub card: Color32,
    /// Normal boundary/connector color.
    pub border: Color32,
    /// Text color.
    pub text: Color32,
    /// Selected boundary and marquee color.
    pub selection: Color32,
    /// Invalid transient geometry color.
    pub error: Color32,
    /// Dotted grid color.
    pub grid: Color32,
    /// Screen-independent base text size; zoom is applied before shaping.
    pub font_size: f32,
}
impl CanvasViewStyle {
    /// Use the host's current egui appearance.
    pub fn from_visuals(visuals: &egui::Visuals) -> Self {
        Self {
            background: visuals.extreme_bg_color,
            card: visuals.panel_fill,
            border: visuals.widgets.noninteractive.bg_stroke.color,
            text: visuals.text_color(),
            selection: visuals.selection.bg_fill,
            error: visuals.error_fg_color,
            grid: visuals.weak_text_color().gamma_multiply(0.25),
            font_size: 14.0,
        }
    }
}
/// Native input/presentation contract supplied by the host each frame.
#[derive(Clone, Debug)]
pub struct CanvasViewConfig {
    /// Stable per-board widget namespace.
    pub id: Id,
    /// Current host theme.
    pub style: CanvasViewStyle,
    /// Current host-localized labels.
    pub labels: CanvasViewLabels,
    /// Whether the host permits editing input in this projection.
    pub interactive: bool,
    /// Block board shortcuts while a host dialog, composer or embedded editor owns input.
    pub shortcuts_blocked: bool,
}
impl CanvasViewConfig {
    /// Create a host-themed configuration; callers replace labels from their locale catalog.
    pub fn new(id: Id, ui: &Ui) -> Self {
        Self {
            id,
            style: CanvasViewStyle::from_visuals(ui.visuals()),
            labels: CanvasViewLabels::default(),
            interactive: true,
            shortcuts_blocked: false,
        }
    }
}
/// Native media/text body result; interactive host controls retain their input ownership.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeBodyResponse {
    /// A child control owns this frame's pointer command at its exact reserved rectangle.
    pub pointer_captured: bool,
    /// A child text/media control owns keyboard commands this frame.
    pub keyboard_captured: bool,
}
/// Authorized host rendering only. Raw file/URL strings never become ambient I/O here.
pub trait CanvasViewHost {
    /// A retained new-card input, rendered only against its exact unchanged base.
    /// The default has no provisional surface; returning an editor grants no edit authority.
    fn pending_text_editor(&self) -> Option<&crate::TextCardEditor> {
        None
    }
    /// Paint a body inside the clipped reserved card rectangle. Media controls can return
    /// input ownership; the adapter will not turn their click into a card drag or shortcut.
    fn paint_body(
        &mut self,
        ui: &mut Ui,
        node: &NodeRecord,
        rect: Rect,
        zoom: f64,
    ) -> NodeBodyResponse;
}
/// One host action at its frozen canonical session identity.
#[derive(Debug)]
pub enum CanvasIntent {
    /// A complete geometry/deletion command; host prepares, checkpoints and acknowledges it.
    Transaction(CanvasTransaction),
    /// Prepare and durably acknowledge a compensating Undo.
    Undo(ContentIdentity),
    /// Prepare and durably acknowledge a compensating Redo.
    Redo(ContentIdentity),
    /// Open a native source editor, keeping this exact expected base.
    EditText {
        /// Frozen base.
        base: ContentIdentity,
        /// Stable text-card identity.
        node_id: String,
    },
    /// Explicit host navigation; it does not grant resource access.
    OpenNode {
        /// Frozen base.
        base: ContentIdentity,
        /// Stable file/link card identity.
        node_id: String,
    },
    /// Ask the host to create a text card using a fresh identity at this world location.
    CreateText {
        /// Frozen base.
        base: ContentIdentity,
        /// Pointer world position.
        position: WorldPoint,
    },
    /// Capture selected complete records using the host clipboard capability.
    Copy {
        /// Frozen base.
        base: ContentIdentity,
        /// Stable current selection.
        selection: Selection,
    },
    /// Capture clipboard bytes before preparing the guarded removal.
    Cut {
        /// Frozen base.
        base: ContentIdentity,
        /// Stable current selection.
        selection: Selection,
    },
    /// Paste from the host's command-boundary bundle; no internal cache shadows OS data.
    Paste {
        /// Frozen base.
        base: ContentIdentity,
        /// Viewport-centered insertion position.
        position: WorldPoint,
        /// Plain text already delivered in this egui command, if present.
        text: Option<String>,
    },
    /// Allocate a complete fresh mapping and prepare duplication through the shared model.
    Duplicate {
        /// Frozen base.
        base: ContentIdentity,
        /// Explicit stable current node IDs.
        node_ids: Vec<String>,
    },
}
/// Observed reserved node geometry from this actual frame, suitable for host review evidence.
#[derive(Clone, Debug)]
pub struct VisibleNodeGeometry {
    /// Stable node ID.
    pub node_id: String,
    /// Allocated visible card rectangle in screen points, before clipping.
    pub reserved: Rect,
    /// Child-body clip rectangle, intersected with the actual viewport.
    pub body_clip: Rect,
}
/// One actually painted connector; hit testing uses these same screen-space segments.
#[derive(Clone, Debug)]
pub struct VisibleEdgeGeometry {
    /// Stable canonical edge identity.
    pub edge_id: String,
    /// Bounded rounded route, after relative-origin world-to-screen conversion.
    pub points: Vec<Pos2>,
    /// Source-compatible location for a host-owned label or contextual editor trigger.
    pub label_anchor: Pos2,
    /// Closed source arrow triangle when the canonical marker requests it.
    pub source_arrow: Option<[Pos2; 3]>,
    /// Closed target arrow triangle, including the standard omitted-marker default.
    pub target_arrow: Option<[Pos2; 3]>,
}
/// A native frame's typed intents and observed geometry. This is not a persistence receipt.
pub struct CanvasViewOutput {
    /// Viewport focus/pointer response.
    pub response: egui::Response,
    /// Completed user actions, each carrying its exact session base.
    pub intents: Vec<CanvasIntent>,
    /// Explicit input/geometry failures; hosts present an actionable message.
    pub errors: Vec<CanvasError>,
    /// Actual allocated viewport in screen points.
    pub viewport: Rect,
    /// Actual current-frame visible card allocations.
    pub nodes: Vec<VisibleNodeGeometry>,
    /// Actual visible connector paths and label positions, shared with pointer hit testing.
    pub edges: Vec<VisibleEdgeGeometry>,
}
struct MoveGesture {
    base: ContentIdentity,
    start: WorldPoint,
    delta: WorldPoint,
    selected: Vec<String>,
    expanded: BTreeSet<String>,
}
struct ResizeGesture {
    base: ContentIdentity,
    node_id: String,
    expected: WorldRect,
    replacement: WorldRect,
    start: WorldPoint,
}
enum Gesture {
    Move(MoveGesture),
    Resize(ResizeGesture),
    Marquee {
        base: ContentIdentity,
        start: WorldPoint,
        current: WorldPoint,
        original: Selection,
    },
    Pan,
}
/// Disposable native camera/selection/gesture state over a host-owned canonical session.
#[derive(Default)]
pub struct CanvasView {
    camera: Camera,
    selection: Selection,
    base: Option<ContentIdentity>,
    gesture: Option<Gesture>,
    last_size: Option<[u32; 2]>,
}
impl CanvasView {
    /// Current camera; the host may retain it per board.
    pub fn camera(&self) -> Camera {
        self.camera
    }
    /// Replace a previously validated camera without changing canonical data.
    pub fn set_camera(&mut self, camera: Camera) {
        self.gesture = None;
        self.camera = camera;
    }
    /// Current nonauthoritative stable selection.
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    /// Select explicit stable identities; absent IDs are removed on the next frame.
    pub fn set_selection(&mut self, selection: Selection) {
        self.gesture = None;
        self.selection = selection;
    }
    /// Cancel uncommitted input on project/view/dialog lifecycle changes.
    pub fn cancel_transient(&mut self) {
        self.gesture = None;
    }
    /// Whether a geometry gesture is waiting for completion.
    pub fn has_gesture(&self) -> bool {
        self.gesture.is_some()
    }
    /// Fit all or selected node bounds using the actual host viewport size.
    pub fn fit(
        &mut self,
        session: &CanvasSession,
        size: Vec2,
        selected: bool,
    ) -> craft_canvas_model::Result<()> {
        let bounds = session
            .document()
            .nodes()
            .iter()
            .filter(|node| !selected || self.selection.nodes.contains(node.id()))
            .map(|node| node.bounds())
            .reduce(WorldRect::union);
        if let Some(bounds) = bounds {
            self.camera.fit(bounds, point(size.x, size.y), 32.0)?;
        }
        self.gesture = None;
        Ok(())
    }
    /// Paint and operate the native board. Pointer previews are disposable; completed actions
    /// emit typed transactions and never silently mutate the host's canonical document.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        session: &CanvasSession,
        dialect: &dyn CanvasDialect,
        host: &mut dyn CanvasViewHost,
        config: &CanvasViewConfig,
    ) -> CanvasViewOutput {
        let (viewport, response) = ui.allocate_exact_size(
            ui.available_size_before_wrap().max(Vec2::splat(1.0)),
            Sense::click_and_drag(),
        );
        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Other, config.interactive, &config.labels.canvas)
        });
        let size = point(viewport.width(), viewport.height());
        let mut intents = vec![];
        let mut errors = vec![];
        let dimensions = [viewport.width().to_bits(), viewport.height().to_bits()];
        if self.base != Some(session.identity())
            || self.last_size.is_some_and(|old| old != dimensions)
        {
            self.gesture = None;
            if self
                .base
                .is_some_and(|old| !old.same_session(session.identity()))
            {
                self.selection = Selection::default();
            }
            self.selection.retain_existing(session.document());
            self.base = Some(session.identity());
        }
        self.last_size = Some(dimensions);
        let focused = ui.input(|input| input.focused);
        if !focused
            || !ui.is_enabled()
            || !config.interactive
            || config.shortcuts_blocked
            || ui.input(|input| input.key_pressed(Key::Escape))
        {
            self.gesture = None;
        }
        ui.painter()
            .rect_filled(viewport, 0.0, config.style.background);
        self.paint_grid(ui, viewport, config.style.grid);
        let painted_edges = self.paint_edges(ui, viewport, session, config, &mut errors);
        let painter = ui.painter().with_clip_rect(viewport);
        let mut observed = vec![];
        let mut pointer_captured = false;
        let mut keyboard_captured = false;
        let mut node_focus = false;
        let mut double_clicked = None;
        let world_view = world_rect(self.camera, viewport);
        let pending = if config.interactive {
            None
        } else {
            host.pending_text_editor()
                .and_then(|editor| editor.pending_node(session))
        };
        for node in session
            .document()
            .nodes()
            .iter()
            .map(AsRef::as_ref)
            .chain(pending.iter().map(AsRef::as_ref))
        {
            let bounds = self.preview_bounds(node);
            if !bounds.intersects(world_view) {
                continue;
            }
            let rect = screen_rect(self.camera, bounds, viewport);
            let body = rect.shrink(10.0_f32.min(rect.width().min(rect.height()) / 4.0));
            let clip = body.intersect(viewport);
            let selected = self.selection.nodes.contains(node.id())
                || pending
                    .as_ref()
                    .is_some_and(|node_pending| node_pending.id() == node.id());
            let color = if bounds.validate(session.limits()).is_err() {
                config.style.error
            } else if selected {
                config.style.selection
            } else {
                config.style.border
            };
            painter.rect_filled(
                rect,
                5.0,
                if dialect.node_policy(node).group == GroupBehavior::Independent {
                    config.style.card
                } else {
                    config.style.card.gamma_multiply(0.5)
                },
            );
            painter.rect_stroke(
                rect,
                5.0,
                Stroke::new(if selected { 2.0 } else { 1.0 }, color),
                StrokeKind::Inside,
            );
            let node_response = ui.interact(
                rect.intersect(viewport),
                config.id.with(("node", node.id())),
                Sense::click_and_drag(),
            );
            node_response.widget_info(|| {
                WidgetInfo::selected(
                    WidgetType::SelectableLabel,
                    config.interactive,
                    selected,
                    if node.body().is_empty() {
                        node.id()
                    } else {
                        node.body()
                    },
                )
            });
            let node_response = if config.interactive
                && !config.shortcuts_blocked
                && ui.is_enabled()
                && matches!(node.kind(), NodeKind::Text)
                && dialect.node_policy(node).movable
            {
                node_response.on_hover_text(&config.labels.edit_text_hint)
            } else {
                node_response
            };
            if node_response.gained_focus() {
                self.selection = Selection {
                    nodes: BTreeSet::from([node.id().into()]),
                    edges: BTreeSet::new(),
                };
            }
            node_focus |= node_response.has_focus();
            if node_response.double_clicked() {
                double_clicked = Some(node.id().to_owned());
            }
            let child = ui
                .scope_builder(
                    egui::UiBuilder::new()
                        .id_salt(("body", node.id()))
                        .max_rect(body),
                    |ui| {
                        ui.set_clip_rect(clip);
                        host.paint_body(ui, node, body, self.camera.zoom())
                    },
                )
                .inner;
            pointer_captured |= child.pointer_captured;
            keyboard_captured |= child.keyboard_captured;
            if selected && self.selection.nodes.len() == 1 && dialect.node_policy(node).resizable {
                painter.rect_filled(
                    Rect::from_center_size(
                        rect.right_bottom() - Vec2::splat(4.0),
                        Vec2::splat(8.0),
                    ),
                    2.0,
                    config.style.selection,
                );
            }
            observed.push(VisibleNodeGeometry {
                node_id: node.id().into(),
                reserved: rect,
                body_clip: clip,
            });
        }
        if pointer_captured || keyboard_captured {
            self.gesture = None;
        }
        if ui.is_enabled()
            && config.interactive
            && focused
            && !config.shortcuts_blocked
            && !pointer_captured
        {
            let pointer = ui.input(|input| input.pointer.interact_pos());
            let delta = ui.input(|input| input.pointer.delta());
            let inside =
                pointer.is_some_and(|p| viewport.contains(p)) && ui.rect_contains_pointer(viewport);
            if inside && self.gesture.is_none() {
                let (zoom, scroll) =
                    ui.input(|input| (input.zoom_delta(), input.smooth_scroll_delta));
                if let Some(pointer) = pointer {
                    let local = point(pointer.x - viewport.left(), pointer.y - viewport.top());
                    if zoom != 1.0 {
                        if let Err(error) = self.camera.zoom_at(f64::from(zoom), local, size) {
                            errors.push(error);
                        }
                    } else if scroll != Vec2::ZERO
                        && let Err(error) = self.camera.pan(point(scroll.x, scroll.y))
                    {
                        errors.push(error);
                    }
                }
                ui.input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
            }
            let pressed = ui.input(|input| input.pointer.primary_pressed());
            let released = ui.input(|input| input.pointer.primary_released());
            let pan = ui.input(|input| {
                input.pointer.button_down(PointerButton::Middle)
                    || (input.key_down(Key::Space) && input.pointer.primary_down())
            });
            if inside && pan && self.gesture.is_none() {
                self.gesture = Some(Gesture::Pan);
            }
            if let Some(Gesture::Pan) = self.gesture {
                if pan {
                    if let Err(error) = self.camera.pan(point(delta.x, delta.y)) {
                        errors.push(error);
                    }
                } else {
                    self.gesture = None;
                }
            }
            if pressed
                && inside
                && !pan
                && let Some(pointer) = pointer
            {
                response.request_focus();
                let world = self.camera.view_to_world(
                    point(pointer.x - viewport.left(), pointer.y - viewport.top()),
                    size,
                );
                let additive = ui.input(|input| input.modifiers.shift || input.modifiers.command);
                if let Some(node) = session.document().hit_node(world) {
                    if additive {
                        if !self.selection.nodes.insert(node.id().into()) {
                            self.selection.nodes.remove(node.id());
                        }
                    } else if !self.selection.nodes.contains(node.id()) {
                        self.selection = Selection {
                            nodes: BTreeSet::from([node.id().into()]),
                            edges: BTreeSet::new(),
                        };
                    }
                    let policy = dialect.node_policy(node);
                    let node_rect = screen_rect(self.camera, node.bounds(), viewport);
                    if self.selection.nodes.len() == 1
                        && self.selection.nodes.contains(node.id())
                        && policy.resizable
                        && pointer.distance(node_rect.right_bottom()) <= 14.0
                    {
                        self.gesture = Some(Gesture::Resize(ResizeGesture {
                            base: session.identity(),
                            node_id: node.id().into(),
                            expected: node.bounds(),
                            replacement: node.bounds(),
                            start: world,
                        }));
                    } else if policy.movable && self.selection.nodes.contains(node.id()) {
                        let selected: Vec<_> = self.selection.nodes.iter().cloned().collect();
                        let mut work = WorkControl::new(session.limits().max_work_units, None);
                        match session
                            .document()
                            .expand_move_selection(&selected, dialect, &mut work)
                        {
                            Ok(expanded) => {
                                self.gesture = Some(Gesture::Move(MoveGesture {
                                    base: session.identity(),
                                    start: world,
                                    delta: WorldPoint::default(),
                                    selected,
                                    expanded,
                                }))
                            }
                            Err(error) => errors.push(error),
                        }
                    }
                } else if let Some(edge) = hit_edge(&painted_edges, pointer) {
                    if additive {
                        if !self.selection.edges.insert(edge.edge_id.clone()) {
                            self.selection.edges.remove(&edge.edge_id);
                        }
                    } else {
                        self.selection = Selection {
                            nodes: BTreeSet::new(),
                            edges: BTreeSet::from([edge.edge_id.clone()]),
                        };
                    }
                    self.gesture = None;
                } else {
                    let original = if additive {
                        self.selection.clone()
                    } else {
                        Selection::default()
                    };
                    self.selection = original.clone();
                    self.gesture = Some(Gesture::Marquee {
                        base: session.identity(),
                        start: world,
                        current: world,
                        original,
                    });
                }
            }
            if let Some(pointer) = pointer {
                let world = self.camera.view_to_world(
                    point(pointer.x - viewport.left(), pointer.y - viewport.top()),
                    size,
                );
                match &mut self.gesture {
                    Some(Gesture::Move(gesture)) => {
                        gesture.delta = WorldPoint {
                            x: (world.x - gesture.start.x).round(),
                            y: (world.y - gesture.start.y).round(),
                        }
                    }
                    Some(Gesture::Resize(gesture)) => {
                        gesture.replacement.width = (gesture.expected.width + world.x
                            - gesture.start.x)
                            .round()
                            .clamp(
                                session.limits().min_dimension,
                                session.limits().max_dimension,
                            );
                        gesture.replacement.height = (gesture.expected.height + world.y
                            - gesture.start.y)
                            .round()
                            .clamp(
                                session.limits().min_dimension,
                                session.limits().max_dimension,
                            );
                    }
                    Some(Gesture::Marquee {
                        start,
                        current,
                        original,
                        ..
                    }) => {
                        *current = world;
                        let bounds = between(*start, *current);
                        self.selection = original.clone();
                        self.selection.nodes.extend(
                            session
                                .document()
                                .nodes()
                                .iter()
                                .filter(|node| bounds.intersects(node.bounds()))
                                .map(|node| node.id().to_owned()),
                        );
                        painter.rect_stroke(
                            screen_rect(self.camera, bounds, viewport),
                            0.0,
                            Stroke::new(1.0, config.style.selection),
                            StrokeKind::Inside,
                        );
                    }
                    _ => {}
                }
            }
            if released && let Some(gesture) = self.gesture.take() {
                match gesture {
                    Gesture::Move(gesture) if gesture.delta != WorldPoint::default() => intents
                        .push(CanvasIntent::Transaction(CanvasTransaction::new(
                            gesture.base,
                            vec![CanvasCommand::Translate {
                                node_ids: gesture.selected,
                                delta: gesture.delta,
                            }],
                        ))),
                    Gesture::Resize(gesture) if gesture.replacement != gesture.expected => intents
                        .push(CanvasIntent::Transaction(CanvasTransaction::new(
                            gesture.base,
                            vec![CanvasCommand::SetGeometry {
                                node_id: gesture.node_id,
                                expected: gesture.expected,
                                replacement: gesture.replacement,
                            }],
                        ))),
                    Gesture::Marquee { base, .. } if base != session.identity() => {
                        self.selection.retain_existing(session.document())
                    }
                    _ => {}
                }
            }
            if let Some(id) = double_clicked
                && let Some(node) = session.document().node(&id)
            {
                if matches!(node.kind(), NodeKind::Text) {
                    intents.push(CanvasIntent::EditText {
                        base: session.identity(),
                        node_id: id,
                    });
                } else if matches!(node.kind(), NodeKind::File | NodeKind::Link) {
                    intents.push(CanvasIntent::OpenNode {
                        base: session.identity(),
                        node_id: id,
                    });
                }
            }
            if response.double_clicked()
                && let Some(pointer) = pointer
            {
                let world = self.camera.view_to_world(
                    point(pointer.x - viewport.left(), pointer.y - viewport.top()),
                    size,
                );
                if session.document().hit_node(world).is_none()
                    && hit_edge(&painted_edges, pointer).is_none()
                {
                    intents.push(CanvasIntent::CreateText {
                        base: session.identity(),
                        position: world,
                    });
                }
            }
            if !keyboard_captured && (response.has_focus() || node_focus) {
                self.keyboard(ui, session, dialect, response.id, config.id, &mut intents);
            }
        }
        CanvasViewOutput {
            response,
            intents,
            errors,
            viewport,
            nodes: observed,
            edges: painted_edges,
        }
    }
    fn keyboard(
        &mut self,
        ui: &mut Ui,
        session: &CanvasSession,
        dialect: &dyn CanvasDialect,
        canvas_id: Id,
        node_namespace: Id,
        intents: &mut Vec<CanvasIntent>,
    ) {
        let base = session.identity();
        if ui.is_enabled()
            && self.gesture.is_none()
            && self.selection.nodes.len() == 1
            && self.selection.edges.is_empty()
            && let Some(id) = self.selection.nodes.first()
            && let Some(node) = session.document().node(id)
            && matches!(node.kind(), NodeKind::Text)
            && dialect.node_policy(node).movable
            && ui.memory(|memory| {
                let focus = memory.focused();
                focus == Some(canvas_id) || focus == Some(node_namespace.with(("node", node.id())))
            })
            && ui.input(|input| {
                input.focused
                    && !input.events.iter().any(|event| matches!(event, egui::Event::Ime(_)))
            })
            && ui.input_mut(|input| {
                // Logical shortcut matching ignores extra Shift/Alt modifiers.
                let event = input.events.iter().position(|event| matches!(event,
                    egui::Event::Key { key: Key::Enter, pressed: true, repeat: false, modifiers, .. }
                        if modifiers.is_none()));
                if let Some(event) = event {
                    input.events.remove(event);
                    true
                } else {
                    false
                }
            })
        {
            intents.push(CanvasIntent::EditText {
                base,
                node_id: node.id().into(),
            });
            return;
        }
        let command = egui::Modifiers::COMMAND;
        let redo = egui::Modifiers {
            shift: true,
            ..command
        };
        if ui.input_mut(|input| {
            input.consume_key(redo, Key::Z) || input.consume_key(command, Key::Y)
        }) {
            intents.push(CanvasIntent::Redo(base));
        } else if ui.input_mut(|input| input.consume_key(command, Key::Z)) {
            intents.push(CanvasIntent::Undo(base));
        }
        if ui.input_mut(|input| input.consume_key(command, Key::A)) {
            self.selection.nodes = session
                .document()
                .nodes()
                .iter()
                .map(|node| node.id().into())
                .collect();
            self.selection.edges = session
                .document()
                .edges()
                .iter()
                .map(|edge| edge.id().into())
                .collect();
        }
        if ui.input_mut(|input| input.consume_key(command, Key::D)) {
            intents.push(CanvasIntent::Duplicate {
                base,
                node_ids: self.selection.nodes.iter().cloned().collect(),
            });
        }
        if ui.input(|input| input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace))
            && (!self.selection.nodes.is_empty() || !self.selection.edges.is_empty())
        {
            intents.push(CanvasIntent::Transaction(CanvasTransaction::new(
                base,
                vec![CanvasCommand::RemoveSelection(self.selection.clone())],
            )));
        }
        let events = ui.input(|input| input.events.clone());
        for event in events {
            match event {
                egui::Event::Copy => intents.push(CanvasIntent::Copy {
                    base,
                    selection: self.selection.clone(),
                }),
                egui::Event::Cut => intents.push(CanvasIntent::Cut {
                    base,
                    selection: self.selection.clone(),
                }),
                egui::Event::Paste(text) => intents.push(CanvasIntent::Paste {
                    base,
                    position: self.camera.center(),
                    text: Some(text),
                }),
                _ => {}
            }
        }
    }
    fn preview_bounds(&self, node: &NodeRecord) -> WorldRect {
        match &self.gesture {
            Some(Gesture::Move(gesture)) if gesture.expanded.contains(node.id()) => {
                node.bounds().translated(gesture.delta)
            }
            Some(Gesture::Resize(gesture)) if gesture.node_id == node.id() => gesture.replacement,
            _ => node.bounds(),
        }
    }
    fn paint_edges(
        &self,
        ui: &Ui,
        viewport: Rect,
        session: &CanvasSession,
        config: &CanvasViewConfig,
        errors: &mut Vec<CanvasError>,
    ) -> Vec<VisibleEdgeGeometry> {
        let clip = viewport.intersect(ui.clip_rect());
        if !clip.is_positive() {
            return Vec::new();
        }
        let painter = ui.painter().with_clip_rect(clip);
        let world_view = world_rect(self.camera, viewport.expand(12.0));
        let mut observed = Vec::new();
        let mut work = WorkControl::new(session.limits().max_work_units, None);
        for edge in session.document().edges() {
            let derived = (|| -> craft_canvas_model::Result<Option<VisibleEdgeGeometry>> {
                work.charge(1)?;
                let from = session
                    .document()
                    .node(edge.from_node())
                    .ok_or_else(|| CanvasError::NotFound(edge.from_node().into()))?;
                let to = session
                    .document()
                    .node(edge.to_node())
                    .ok_or_else(|| CanvasError::NotFound(edge.to_node().into()))?;
                // Loose source handles choose the first (Top) handle when a legacy side
                // is omitted. This fallback is presentation only, never a JSON rewrite.
                let from_side = edge.from_side().unwrap_or(Side::Top);
                let to_side = edge.to_side().unwrap_or(Side::Top);
                let route = if edge.from_node() == edge.to_node() {
                    SmoothStepConnector::loop_around(self.preview_bounds(from), from_side, to_side)?
                } else {
                    SmoothStepConnector::between(
                        from_side.point(self.preview_bounds(from)),
                        from_side,
                        to_side.point(self.preview_bounds(to)),
                        to_side,
                    )?
                };
                if !world_view.intersects(route.bounds()) {
                    return Ok(None);
                }
                let points: Vec<_> = route
                    .flatten(0.35 / self.camera.zoom(), 64, &mut work)?
                    .into_iter()
                    .map(|point| screen(self.camera, point, viewport))
                    .collect();
                let marker_size = (8.0 * self.camera.zoom()) as f32;
                let source_arrow = (edge.from_end() == ConnectorEnd::Arrow)
                    .then(|| arrow(&points, false, marker_size))
                    .flatten();
                let target_arrow = (edge.to_end() == ConnectorEnd::Arrow)
                    .then(|| arrow(&points, true, marker_size))
                    .flatten();
                Ok(Some(VisibleEdgeGeometry {
                    edge_id: edge.id().into(),
                    points,
                    label_anchor: screen(self.camera, route.label_anchor(), viewport),
                    source_arrow,
                    target_arrow,
                }))
            })();
            match derived {
                Ok(Some(geometry)) => {
                    let selected = self.selection.edges.contains(edge.id());
                    let color = if selected {
                        config.style.selection
                    } else {
                        config.style.border
                    };
                    let stroke = Stroke::new(
                        (if selected { 3.0 } else { 2.0 }) * self.camera.zoom() as f32,
                        color,
                    );
                    painter.add(egui::Shape::line(geometry.points.clone(), stroke));
                    for triangle in [geometry.source_arrow, geometry.target_arrow]
                        .into_iter()
                        .flatten()
                    {
                        painter.add(egui::Shape::convex_polygon(
                            triangle.to_vec(),
                            color,
                            Stroke::NONE,
                        ));
                    }
                    observed.push(geometry);
                }
                Ok(None) => {}
                Err(error) => {
                    // A bounded frame reports one actionable failure, not one per remaining edge.
                    errors.push(error);
                    break;
                }
            }
        }
        observed
    }
    fn paint_grid(&self, ui: &Ui, viewport: Rect, color: Color32) {
        let spacing = (24.0 * self.camera.zoom()).max(12.0);
        let x0 = f64::from(viewport.left())
            + (f64::from(viewport.width()) / 2.0 - self.camera.center().x * self.camera.zoom())
                .rem_euclid(spacing);
        let y0 = f64::from(viewport.top())
            + (f64::from(viewport.height()) / 2.0 - self.camera.center().y * self.camera.zoom())
                .rem_euclid(spacing);
        let cols = (f64::from(viewport.width()) / spacing).ceil() as usize;
        let rows = (f64::from(viewport.height()) / spacing).ceil() as usize;
        let painter = ui.painter().with_clip_rect(viewport);
        for x in 0..=cols.min(512) {
            for y in 0..=rows.min(512) {
                painter.circle_filled(
                    Pos2::new(
                        (x0 + x as f64 * spacing) as f32,
                        (y0 + y as f64 * spacing) as f32,
                    ),
                    0.8,
                    color,
                );
            }
        }
    }
}
fn hit_edge(edges: &[VisibleEdgeGeometry], pointer: Pos2) -> Option<&VisibleEdgeGeometry> {
    // Nodes are tested first by the caller; among edges, the last painted line wins.
    edges.iter().rev().find(|edge| {
        edge.points.windows(2).any(|segment| {
            distance_to_segment(
                point(pointer.x, pointer.y),
                point(segment[0].x, segment[0].y),
                point(segment[1].x, segment[1].y),
            ) <= 6.0
        }) || [edge.source_arrow, edge.target_arrow]
            .into_iter()
            .flatten()
            .any(|triangle| {
                let cross = |a: Pos2, b: Pos2| {
                    let line = b - a;
                    let offset = pointer - a;
                    line.x * offset.y - line.y * offset.x
                };
                let sides = [
                    cross(triangle[0], triangle[1]),
                    cross(triangle[1], triangle[2]),
                    cross(triangle[2], triangle[0]),
                ];
                sides.iter().all(|side| *side >= 0.0) || sides.iter().all(|side| *side <= 0.0)
            })
    })
}
fn arrow(points: &[Pos2], at_end: bool, size: f32) -> Option<[Pos2; 3]> {
    let (tip, neighbor) = if at_end {
        let tip = *points.last()?;
        (tip, *points.iter().rev().find(|point| **point != tip)?)
    } else {
        let tip = *points.first()?;
        (tip, *points.iter().find(|point| **point != tip)?)
    };
    let direction = (neighbor - tip).normalized();
    let center = tip + direction * size;
    let perpendicular = Vec2::new(-direction.y, direction.x) * size * 0.5;
    Some([tip, center + perpendicular, center - perpendicular])
}
fn point(x: f32, y: f32) -> WorldPoint {
    WorldPoint {
        x: f64::from(x),
        y: f64::from(y),
    }
}
fn screen(camera: Camera, world: WorldPoint, viewport: Rect) -> Pos2 {
    let local = camera.world_to_view(world, point(viewport.width(), viewport.height()));
    Pos2::new(
        viewport.left() + local.x as f32,
        viewport.top() + local.y as f32,
    )
}
fn screen_rect(camera: Camera, rect: WorldRect, viewport: Rect) -> Rect {
    Rect::from_min_max(
        screen(
            camera,
            WorldPoint {
                x: rect.x,
                y: rect.y,
            },
            viewport,
        ),
        screen(
            camera,
            WorldPoint {
                x: rect.x + rect.width,
                y: rect.y + rect.height,
            },
            viewport,
        ),
    )
}
fn world_rect(camera: Camera, viewport: Rect) -> WorldRect {
    let size = point(viewport.width(), viewport.height());
    let min = camera.view_to_world(WorldPoint::default(), size);
    WorldRect {
        x: min.x,
        y: min.y,
        width: size.x / camera.zoom(),
        height: size.y / camera.zoom(),
    }
}
fn between(a: WorldPoint, b: WorldPoint) -> WorldRect {
    WorldRect {
        x: a.x.min(b.x),
        y: a.y.min(b.y),
        width: (a.x - b.x).abs(),
        height: (a.y - b.y).abs(),
    }
}
/// A simple inert body for examples; real GFM/resource rendering is an explicit host adapter.
pub struct PlainTextBody;
impl CanvasViewHost for PlainTextBody {
    fn paint_body(
        &mut self,
        ui: &mut Ui,
        node: &NodeRecord,
        rect: Rect,
        zoom: f64,
    ) -> NodeBodyResponse {
        ui.painter().text(
            rect.left_top(),
            Align2::LEFT_TOP,
            node.body(),
            FontId::proportional((14.0 * zoom as f32).clamp(8.0, 42.0)),
            ui.visuals().text_color(),
        );
        NodeBodyResponse::default()
    }
}
