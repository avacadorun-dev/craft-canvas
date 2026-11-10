//! Real egui event regressions over synthetic records; native OS acceptance is separate.
use std::collections::BTreeSet;

use craft_canvas_egui::*;
use craft_canvas_model::*;
use egui::{Event, Id, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const NODES: &str = r#"[
    {"id":"a","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"Synthetic A"},
    {"id":"b","type":"text","x":100,"y":0,"width":160,"height":120,"text":"Synthetic B"}
]"#;
const EDGES: &str =
    r#"[{"id":"edge","fromNode":"a","fromSide":"right","toNode":"b","toSide":"left"}]"#;

#[derive(Default)]
struct Policies {
    hidden_nodes: BTreeSet<&'static str>,
    nonselectable_nodes: BTreeSet<&'static str>,
    unfocusable_nodes: BTreeSet<&'static str>,
    immovable_nodes: BTreeSet<&'static str>,
    unresizable_nodes: BTreeSet<&'static str>,
    protected_nodes: BTreeSet<&'static str>,
    hidden_edges: BTreeSet<&'static str>,
    nonselectable_edges: BTreeSet<&'static str>,
}
impl CanvasDialect for Policies {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        let mut policy = StandardDialect.node_policy(node);
        policy.movable &= !self.immovable_nodes.contains(node.id());
        policy.resizable &= !self.unresizable_nodes.contains(node.id());
        policy
    }
    fn node_interaction(&self, node: &NodeRecord) -> NodeInteractionPolicy {
        NodeInteractionPolicy {
            selectable: !self.nonselectable_nodes.contains(node.id()),
            deletable: !self.protected_nodes.contains(node.id()),
            focusable: !self.unfocusable_nodes.contains(node.id()),
            hidden: self.hidden_nodes.contains(node.id()),
        }
    }
    fn edge_interaction(&self, edge: &EdgeRecord) -> EdgeInteractionPolicy {
        EdgeInteractionPolicy {
            selectable: !self.nonselectable_edges.contains(edge.id()),
            hidden: self.hidden_edges.contains(edge.id()),
            ..EdgeInteractionPolicy::default()
        }
    }
}

#[derive(Default)]
struct Host {
    bodies: Vec<String>,
    allocations: usize,
    field: bool,
    text: String,
}
impl CanvasViewHost for Host {
    fn allocate_edge_id(
        &mut self,
        _base: ContentIdentity,
        _origin: &PortEndpoint,
    ) -> Result<Option<String>> {
        self.allocations += 1;
        Ok(Some(format!("created-{}", self.allocations)))
    }
    fn paint_body(
        &mut self,
        ui: &mut egui::Ui,
        node: &NodeRecord,
        _rect: Rect,
        _zoom: f64,
    ) -> NodeBodyResponse {
        self.bodies.push(node.id().into());
        if self.field && node.id() == "a" {
            let response = ui
                .add(egui::TextEdit::multiline(&mut self.text).id(Id::new("policy-native-field")));
            NodeBodyResponse {
                keyboard_captured: response.has_focus(),
                pointer_captured: response.is_pointer_button_down_on(),
            }
        } else {
            NodeBodyResponse::default()
        }
    }
}

struct Harness {
    ctx: egui::Context,
    view: CanvasView,
    session: CanvasSession,
    policies: Policies,
    host: Host,
    mode: SelectionMode,
    connections: bool,
    time: f64,
}
impl Harness {
    fn new(nodes: &str, edges: &str) -> Result<Self> {
        let limits = CanvasLimits::default();
        let raw = format!(r#"{{"nodes":{nodes},"edges":{edges}}}"#);
        Ok(Self {
            ctx: egui::Context::default(),
            view: CanvasView::default(),
            session: CanvasSession::new(CanvasDocument::parse(&raw, &limits)?, limits)?,
            policies: Policies::default(),
            host: Host::default(),
            mode: SelectionMode::default(),
            connections: false,
            time: 0.0,
        })
    }
    fn frame(&mut self, events: Vec<Event>) -> Result<CanvasViewOutput> {
        self.time += 1.0;
        self.host.bodies.clear();
        let mut output = None;
        let modifiers = events
            .iter()
            .find_map(|event| match event {
                Event::Key { modifiers, .. } | Event::PointerButton { modifiers, .. } => {
                    Some(*modifiers)
                }
                _ => None,
            })
            .unwrap_or_default();
        let mut full = self.ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 680.))),
                focused: true,
                time: Some(self.time),
                modifiers,
                events,
                ..RawInput::default()
            },
            |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    let mut config = CanvasViewConfig::new(Id::new("policy-board"), ui);
                    config.selection_mode = self.mode;
                    config.connections = self.connections.then(ConnectionOptions::default);
                    output = Some(self.view.show(
                        ui,
                        &self.session,
                        &self.policies,
                        &mut self.host,
                        &config,
                    ));
                });
            },
        );
        // The harness has no native renderer. Input behavior is the evidence here.
        full.textures_delta.clear();
        output.ok_or_else(|| CanvasError::Invalid("missing policy frame".into()))
    }
    fn apply(&mut self, transaction: CanvasTransaction) -> Result<()> {
        let prepared = self.session.prepare(transaction, &self.policies)?;
        self.session.apply(prepared)?;
        Ok(())
    }
}
fn selection(nodes: &[&str], edges: &[&str]) -> Selection {
    Selection {
        nodes: nodes.iter().map(|id| (*id).into()).collect(),
        edges: edges.iter().map(|id| (*id).into()).collect(),
    }
}
fn pointer(pos: Pos2, pressed: bool) -> Vec<Event> {
    pointer_modified(pos, pressed, Modifiers::NONE)
}
fn pointer_modified(pos: Pos2, pressed: bool, modifiers: Modifiers) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers,
        },
    ]
}
fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn node_rect(output: &CanvasViewOutput, id: &str) -> Result<Rect> {
    output
        .nodes
        .iter()
        .find(|node| node.node_id == id)
        .map(|node| node.reserved)
        .ok_or_else(|| CanvasError::NotFound(id.into()))
}
fn port(output: &CanvasViewOutput, id: &str, side: Side) -> Result<Pos2> {
    output
        .ports
        .iter()
        .find(|port| {
            port.endpoint.node_id == id && port.endpoint.binding == PortBinding::Legacy(Some(side))
        })
        .map(|port| port.center)
        .ok_or_else(|| CanvasError::NotFound(format!("{id} {side:?}")))
}
fn transaction(output: CanvasViewOutput) -> Result<CanvasTransaction> {
    if output.intents.len() != 1 {
        return Err(CanvasError::Invalid("expected exactly one intent".into()));
    }
    output
        .intents
        .into_iter()
        .find_map(|intent| match intent {
            CanvasIntent::Transaction(transaction) => Some(transaction),
            _ => None,
        })
        .ok_or_else(|| CanvasError::Invalid("expected transaction".into()))
}

#[test]
fn hidden_records_have_no_native_body_port_route_or_pointer_occlusion() -> TestResult {
    let nodes = r#"[
        {"id":"a","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"A"},
        {"id":"b","type":"text","x":100,"y":0,"width":160,"height":120,"text":"B"},
        {"id":"cover","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"Cover"}
    ]"#;
    let mut h = Harness::new(nodes, EDGES)?;
    h.connections = true;
    let original = h.session.document().raw_json().to_owned();
    let initial = h.frame(vec![])?;
    let a = node_rect(&initial, "a")?.center();
    let b = node_rect(&initial, "b")?.center();
    h.view
        .set_selection(selection(&["a", "b", "cover"], &["edge"]));
    h.policies.hidden_nodes.extend(["b", "cover"]);
    let hidden = h.frame(vec![])?;
    assert_eq!(h.host.bodies, ["a"]);
    assert_eq!(hidden.nodes.len(), 1);
    assert!(hidden.ports.iter().all(|port| port.endpoint.node_id == "a"));
    assert_eq!(hidden.ports.len(), 4);
    assert!(hidden.edges.is_empty());
    assert!(hidden.reconnect_handles.is_empty());
    assert!(hidden.errors.is_empty());
    assert_eq!(h.view.selection(), &selection(&["a"], &[]));
    h.view.set_selection(Selection::default());
    h.frame(pointer(a, true))?;
    h.frame(pointer(a, false))?;
    assert_eq!(h.view.selection(), &selection(&["a"], &[]));
    h.frame(pointer(b, true))?;
    h.frame(pointer(b, false))?;
    assert_eq!(h.view.selection(), &Selection::default());
    assert_eq!(h.session.document().raw_json(), original);
    assert_eq!(h.session.document().nodes().len(), 3);
    Ok(())
}

#[test]
fn nonselectable_topmost_node_moves_independently_of_existing_selection() -> TestResult {
    let nodes = r#"[
        {"id":"under","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"Under"},
        {"id":"a","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"A"},
        {"id":"b","type":"text","x":100,"y":0,"width":160,"height":120,"text":"B"}
    ]"#;
    let mut h = Harness::new(nodes, "[]")?;
    h.policies.nonselectable_nodes.insert("a");
    h.view.set_selection(selection(&["b"], &[]));
    let initial = h.frame(vec![])?;
    let start = node_rect(&initial, "a")?.center();
    let end = start + Vec2::new(30., 20.);
    h.frame(pointer(start, true))?;
    let released = h.frame(pointer(end, false))?;
    assert_eq!(h.view.selection(), &selection(&["b"], &[]));
    assert!(released.selection_change.is_none());
    h.apply(transaction(released)?)?;
    assert_eq!(h.session.document().node("a").ok_or("a")?.bounds().x, -270.);
    assert_eq!(
        h.session
            .document()
            .node("under")
            .ok_or("under")?
            .bounds()
            .x,
        -300.
    );
    assert_eq!(h.session.document().node("b").ok_or("b")?.bounds().x, 100.);
    assert!(h.frame(vec![])?.selection_change.is_none());
    Ok(())
}

#[test]
fn independent_group_drag_keeps_normal_members_including_initially_hidden_members() -> TestResult {
    let nodes = r#"[
        {"id":"group","type":"group","x":-300,"y":-100,"width":400,"height":300,"label":"Group"},
        {"id":"member","type":"text","x":-220,"y":0,"width":80,"height":60,"text":"Selected"},
        {"id":"hidden","type":"text","x":-80,"y":0,"width":80,"height":60,"text":"Hidden"},
        {"id":"outside","type":"text","x":200,"y":0,"width":80,"height":60,"text":"Outside"}
    ]"#;
    let mut h = Harness::new(nodes, "[]")?;
    h.policies.nonselectable_nodes.insert("group");
    h.policies.hidden_nodes.insert("hidden");
    h.view.set_selection(selection(&["member", "outside"], &[]));
    let initial = h.frame(vec![])?;
    let start = node_rect(&initial, "group")?.left_top() + Vec2::splat(20.);
    h.frame(pointer(start, true))?;
    let released = h.frame(pointer(start + Vec2::new(20., 0.), false))?;
    h.apply(transaction(released)?)?;
    assert_eq!(
        h.session
            .document()
            .node("member")
            .ok_or("member")?
            .bounds()
            .x,
        -200.
    );
    assert_eq!(
        h.session
            .document()
            .node("hidden")
            .ok_or("hidden")?
            .bounds()
            .x,
        -60.
    );
    assert_eq!(
        h.session
            .document()
            .node("outside")
            .ok_or("outside")?
            .bounds()
            .x,
        200.
    );
    assert_eq!(h.view.selection(), &selection(&["member", "outside"], &[]));
    Ok(())
}

#[test]
fn hidden_or_newly_immovable_independent_drag_cancels_without_selection_change() -> TestResult {
    let mut h = Harness::new(NODES, "[]")?;
    h.policies.nonselectable_nodes.insert("a");
    h.view.set_selection(selection(&["b"], &[]));
    let original = h.session.document().raw_json().to_owned();
    let initial = h.frame(vec![])?;
    let start = node_rect(&initial, "a")?.center();
    h.frame(pointer(start, true))?;
    h.frame(vec![Event::PointerMoved(start + Vec2::new(20., 0.))])?;
    h.policies.hidden_nodes.insert("a");
    let hidden = h.frame(vec![])?;
    assert!(!h.view.has_gesture());
    assert!(hidden.intents.is_empty());
    assert!(hidden.selection_change.is_none());
    h.policies.hidden_nodes.clear();
    assert!(
        h.frame(pointer(start + Vec2::new(30., 0.), false))?
            .intents
            .is_empty()
    );
    h.frame(pointer(start, true))?;
    h.policies.immovable_nodes.insert("a");
    assert!(
        h.frame(pointer(start + Vec2::new(30., 0.), false))?
            .intents
            .is_empty()
    );
    assert!(!h.view.has_gesture());
    assert_eq!(h.session.document().raw_json(), original);
    assert_eq!(h.view.selection(), &selection(&["b"], &[]));
    Ok(())
}

#[test]
fn resize_permission_loss_and_selection_filtering_cancel_captured_geometry() -> TestResult {
    let mut h = Harness::new(NODES, "[]")?;
    h.view.set_selection(selection(&["a"], &[]));
    let initial = h.frame(vec![])?;
    let corner = node_rect(&initial, "a")?.right_bottom() - Vec2::splat(4.);
    h.frame(pointer(corner, true))?;
    h.policies.unresizable_nodes.insert("a");
    assert!(
        h.frame(pointer(corner + Vec2::splat(20.), false))?
            .intents
            .is_empty()
    );
    assert!(!h.view.has_gesture());
    let center = node_rect(&initial, "a")?.center();
    h.frame(pointer(center, true))?;
    h.policies.nonselectable_nodes.insert("a");
    let filtered = h.frame(pointer(center + Vec2::splat(20.), false))?;
    assert!(filtered.intents.is_empty());
    assert_eq!(
        filtered
            .selection_change
            .ok_or("selection change")?
            .selection,
        Selection::default()
    );
    assert!(!h.view.has_gesture());
    assert_eq!(h.session.history_usage().undo_steps, 0);
    Ok(())
}

#[test]
fn nonselectable_edge_blocks_lower_route_and_hidden_edge_reveals_it() -> TestResult {
    let edges = r#"[
        {"id":"edge","fromNode":"a","fromSide":"right","toNode":"b","toSide":"left"},
        {"id":"cover","fromNode":"a","fromSide":"right","toNode":"b","toSide":"left"}
    ]"#;
    let mut h = Harness::new(NODES, edges)?;
    h.policies.nonselectable_edges.insert("cover");
    let initial = h.frame(vec![])?;
    let point = initial.edges[0].label_anchor;
    h.frame(pointer(point, true))?;
    h.frame(pointer(point, false))?;
    assert_eq!(h.view.selection(), &Selection::default());
    assert!(!h.view.has_gesture());
    h.policies.hidden_edges.insert("cover");
    let hidden = h.frame(vec![])?;
    assert_eq!(hidden.edges.len(), 1);
    h.frame(pointer(point, true))?;
    h.frame(pointer(point, false))?;
    assert_eq!(h.view.selection(), &selection(&[], &["edge"]));
    Ok(())
}

#[test]
fn wrapper_focus_policy_never_selects_nonselectable_node_or_steals_native_text_focus() -> TestResult
{
    let mut h = Harness::new(NODES, "[]")?;
    h.policies.nonselectable_nodes.insert("a");
    h.view.set_selection(selection(&["b"], &[]));
    h.frame(vec![])?;
    let wrapper = Id::new("policy-board").with(("node", "a"));
    h.ctx.memory_mut(|memory| memory.request_focus(wrapper));
    h.frame(vec![])?;
    assert_eq!(h.view.selection(), &selection(&["b"], &[]));
    assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(wrapper));
    h.policies.unfocusable_nodes.insert("a");
    h.frame(vec![])?;
    assert_ne!(h.ctx.memory(|memory| memory.focused()), Some(wrapper));
    h.host.field = true;
    h.frame(vec![])?;
    let field = Id::new("policy-native-field");
    h.ctx.memory_mut(|memory| memory.request_focus(field));
    h.frame(vec![Event::Text("日本語 e\u{301}".into())])?;
    assert_eq!(h.host.text, "日本語 e\u{301}");
    assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(field));
    assert_eq!(h.view.selection(), &selection(&["b"], &[]));
    Ok(())
}

#[test]
fn additive_click_and_toggle_survive_wrapper_focus_gain() -> TestResult {
    let mut h = Harness::new(NODES, "[]")?;
    h.view.set_selection(selection(&["a"], &[]));
    let initial = h.frame(vec![])?;
    let b = node_rect(&initial, "b")?.center();
    let wrapper = Id::new("policy-board").with(("node", "b"));
    h.ctx.memory_mut(|memory| memory.request_focus(wrapper));
    h.frame(pointer_modified(b, true, Modifiers::SHIFT))?;
    h.frame(pointer_modified(b, false, Modifiers::SHIFT))?;
    assert_eq!(h.view.selection(), &selection(&["a", "b"], &[]));
    h.ctx.memory_mut(|memory| memory.request_focus(wrapper));
    h.frame(pointer_modified(b, true, Modifiers::COMMAND))?;
    h.frame(pointer_modified(b, false, Modifiers::COMMAND))?;
    assert_eq!(h.view.selection(), &selection(&["a"], &[]));
    assert!(!h.view.has_gesture());
    assert_eq!(h.session.history_usage().undo_steps, 0);
    Ok(())
}

#[test]
fn marquee_modes_and_select_all_share_visible_selectable_policy() -> TestResult {
    let mut h = Harness::new(NODES, EDGES)?;
    h.policies.nonselectable_nodes.insert("b");
    h.policies.nonselectable_edges.insert("edge");
    let initial = h.frame(vec![])?;
    let a = node_rect(&initial, "a")?;
    let start = a.left_top() - Vec2::splat(20.);
    h.frame(pointer(start, true))?;
    h.frame(pointer(a.center(), false))?;
    assert_eq!(h.view.selection(), &selection(&["a"], &[]));
    h.mode = SelectionMode::Contain;
    h.frame(pointer(start, true))?;
    h.frame(pointer(a.center(), false))?;
    assert_eq!(h.view.selection(), &Selection::default());
    h.frame(pointer(start, true))?;
    h.frame(pointer(a.right_bottom() + Vec2::splat(20.), false))?;
    assert_eq!(h.view.selection(), &selection(&["a"], &[]));
    h.view.set_selection(Selection::default());
    let ready = h.frame(vec![])?;
    h.ctx
        .memory_mut(|memory| memory.request_focus(ready.response.id));
    h.frame(vec![key(Key::A, Modifiers::COMMAND)])?;
    assert_eq!(h.view.selection(), &selection(&["a"], &[]));
    h.policies.protected_nodes.insert("a");
    let deletion = transaction(h.frame(vec![key(Key::Delete, Modifiers::NONE)])?)?;
    assert!(
        matches!(h.session.prepare(deletion, &h.policies), Err(CanvasError::NotEditable(id)) if id == "a")
    );
    assert_eq!(h.session.history_usage().undo_steps, 0);
    Ok(())
}

#[test]
fn selection_observation_reports_filtered_changes_once_without_revision_noise() -> TestResult {
    let mut h = Harness::new(NODES, EDGES)?;
    assert!(h.frame(vec![])?.selection_change.is_none());
    h.view.set_selection(selection(&["absent"], &["absent"]));
    assert!(h.frame(vec![])?.selection_change.is_none());
    h.view.set_selection(selection(&["a", "absent"], &[]));
    let changed = h
        .frame(vec![])?
        .selection_change
        .ok_or("changed selection")?;
    assert_eq!(changed.base, h.session.identity());
    assert_eq!(changed.selection, selection(&["a"], &[]));
    assert!(h.frame(vec![])?.selection_change.is_none());
    h.apply(CanvasTransaction::new(
        h.session.identity(),
        vec![CanvasCommand::EditText {
            node_id: "a".into(),
            expected: "Synthetic A".into(),
            text: "Changed synthetic A".into(),
        }],
    ))?;
    assert!(h.frame(vec![])?.selection_change.is_none());
    h.policies.hidden_nodes.insert("a");
    let changed = h
        .frame(vec![])?
        .selection_change
        .ok_or("filtered selection")?;
    assert_eq!(changed.base, h.session.identity());
    assert_eq!(changed.selection, Selection::default());
    assert!(h.frame(vec![])?.selection_change.is_none());
    Ok(())
}

#[test]
fn stationary_connection_capture_cancels_when_an_unselected_endpoint_becomes_hidden() -> TestResult
{
    for hidden in ["a", "b"] {
        let mut h = Harness::new(NODES, "[]")?;
        h.connections = true;
        let initial = h.frame(vec![])?;
        let from = port(&initial, "a", Side::Right)?;
        let to = port(&initial, "b", Side::Left)?;
        h.frame(pointer(from, true))?;
        h.frame(vec![Event::PointerMoved(to)])?;
        h.policies.hidden_nodes.insert(hidden);
        let cancelled = h.frame(vec![])?;
        assert!(matches!(
            cancelled.connection_events.as_slice(),
            [ConnectionEvent::Cancel {
                reason: ConnectionCancelReason::GeometryUnavailable,
                ..
            }]
        ));
        assert!(cancelled.consumed_input.pointer);
        assert!(cancelled.connection_preview.is_none());
        assert!(cancelled.intents.is_empty());
        assert!(cancelled.errors.is_empty());
        assert!(!h.view.has_gesture());
        h.policies.hidden_nodes.clear();
        assert!(h.frame(pointer(to, false))?.intents.is_empty());
        assert_eq!(h.session.history_usage().undo_steps, 0);
    }
    Ok(())
}

#[test]
fn hidden_pending_connection_keeps_receipt_and_acknowledgement_authority() -> TestResult {
    let mut h = Harness::new(NODES, "[]")?;
    h.connections = true;
    let initial = h.frame(vec![])?;
    let from = port(&initial, "a", Side::Right)?;
    let to = port(&initial, "b", Side::Left)?;
    h.frame(pointer(from, true))?;
    let request = transaction(h.frame(pointer(to, false))?)?;
    let prepared = h.session.prepare(request, &h.policies)?;
    h.view.bind_connection_prepared(&prepared)?;
    h.policies.hidden_nodes.insert("b");
    let hidden = h.frame(vec![])?;
    assert!(hidden.connection_preview.is_none());
    assert!(hidden.consumed_input.pointer);
    assert!(hidden.connection_events.is_empty());
    assert!(hidden.intents.is_empty());
    assert_eq!(
        h.view.connection_status(),
        Some(&ConnectionSubmissionState::Awaiting)
    );
    h.policies.hidden_nodes.clear();
    assert!(h.frame(vec![])?.connection_preview.is_some());
    h.policies.hidden_nodes.insert("b");
    h.session.apply(prepared)?;
    h.view.acknowledge_connection(&h.session)?;
    assert!(!h.view.connection_pending());
    assert_eq!(h.session.document().edges().len(), 1);
    Ok(())
}

#[test]
fn hidden_original_reconnect_edge_cancels_capture_but_only_suppresses_pending_preview() -> TestResult
{
    let nodes = r#"[
        {"id":"a","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"A"},
        {"id":"b","type":"text","x":100,"y":0,"width":160,"height":120,"text":"B"},
        {"id":"c","type":"text","x":100,"y":180,"width":160,"height":120,"text":"C"}
    ]"#;
    let mut h = Harness::new(nodes, EDGES)?;
    h.connections = true;
    h.view.set_selection(selection(&[], &["edge"]));
    let initial = h.frame(vec![])?;
    let grip = initial
        .reconnect_handles
        .iter()
        .find(|handle| handle.end == ConnectorEndpointKind::To)
        .ok_or("target grip")?
        .center;
    let target = port(&initial, "c", Side::Left)?;
    h.frame(pointer(grip, true))?;
    h.frame(vec![Event::PointerMoved(target)])?;
    h.policies.hidden_edges.insert("edge");
    let cancelled = h.frame(vec![])?;
    assert!(matches!(
        cancelled.connection_events.as_slice(),
        [ConnectionEvent::Cancel {
            reason: ConnectionCancelReason::GeometryUnavailable,
            ..
        }]
    ));
    assert!(!h.view.has_gesture());
    assert!(h.frame(pointer(target, false))?.intents.is_empty());
    h.policies.hidden_edges.clear();
    h.view.set_selection(selection(&[], &["edge"]));
    h.frame(vec![])?;
    h.frame(pointer(grip, true))?;
    let request = transaction(h.frame(pointer(target, false))?)?;
    let prepared = h.session.prepare(request, &h.policies)?;
    h.view.bind_connection_prepared(&prepared)?;
    h.policies.hidden_edges.insert("edge");
    assert!(h.frame(vec![])?.connection_preview.is_none());
    assert!(h.view.connection_pending());
    h.policies.hidden_edges.clear();
    assert!(h.frame(vec![])?.connection_preview.is_some());
    h.session.apply(prepared)?;
    h.view.acknowledge_connection(&h.session)?;
    assert_eq!(
        h.session.document().edge("edge").ok_or("edge")?.to_node(),
        "c"
    );
    Ok(())
}
