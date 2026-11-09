//! Synthetic RawInput exercises of the actual view; not OS or durable-host acceptance.
use std::collections::BTreeSet;

use craft_canvas_egui::*;
use craft_canvas_model::*;
use egui::{Event, Id, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn session(legacy_edge: bool) -> Result<CanvasSession> {
    let edges = if legacy_edge {
        r#"[{"id":"legacy","fromNode":"a","fromSide":null,"toNode":"b","toSide":"","extra":{"keep":1.2300}}]"#
    } else {
        "[]"
    };
    let raw = format!(
        r#"{{"nodes":[
            {{"id":"a","type":"text","x":-300,"y":0,"width":160,"height":120,"text":"Synthetic source",
              "craftCanvas":{{"version":1,"ports":[{{"id":"out","kind":"source","side":"right","x":160,"y":60}}]}}}},
            {{"id":"b","type":"text","x":100,"y":0,"width":160,"height":120,"text":"Synthetic target",
              "craftCanvas":{{"version":1,"ports":[{{"id":"in","kind":"target","side":"left","x":0,"y":60}}]}}}}
        ],"edges":{edges}}}"#
    );
    let limits = CanvasLimits::default();
    CanvasSession::new(CanvasDocument::parse(&raw, &limits)?, limits)
}

#[derive(Default)]
struct Host {
    allocations: usize,
    body_capture: bool,
    ownership: CanvasInputOwnership,
    exclusions: Vec<Rect>,
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
    fn input_ownership(&self) -> CanvasInputOwnership {
        self.ownership
    }
    fn paint_body(
        &mut self,
        _ui: &mut egui::Ui,
        _node: &NodeRecord,
        _rect: Rect,
        _zoom: f64,
    ) -> NodeBodyResponse {
        NodeBodyResponse {
            pointer_captured: self.body_capture,
            keyboard_captured: self.body_capture,
        }
    }
}

fn frame(
    ctx: &egui::Context,
    view: &mut CanvasView,
    session: &CanvasSession,
    host: &mut Host,
    events: Vec<Event>,
) -> Result<CanvasViewOutput> {
    let mut output = None;
    let mut rendered = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 680.))),
            focused: true,
            events,
            ..RawInput::default()
        },
        |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut config = CanvasViewConfig::new(Id::new("connection-events"), ui);
                config.connections = Some(ConnectionOptions::default());
                config.input_exclusions.clone_from(&host.exclusions);
                output = Some(view.show(ui, session, &StandardDialect, host, &config));
            });
        },
    );
    rendered.textures_delta.clear();
    rendered.platform_output.commands.clear();
    output.ok_or_else(|| CanvasError::Invalid("missing synthetic connection frame".into()))
}

fn pointer(pos: Pos2, pressed: bool) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        },
    ]
}

fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn named_port(output: &CanvasViewOutput, node: &str) -> Result<VisiblePortGeometry> {
    output
        .ports
        .iter()
        .find(|port| {
            port.endpoint.node_id == node && matches!(port.endpoint.binding, PortBinding::Named(_))
        })
        .cloned()
        .ok_or_else(|| CanvasError::Invalid(format!("missing named port on {node}")))
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
        .ok_or_else(|| CanvasError::Invalid("expected connection transaction".into()))
}

#[test]
fn release_emits_once_and_waits_for_bound_apply_acknowledgement() -> TestResult {
    let mut session = session(false)?;
    let original = session.document().raw_json().to_owned();
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host::default();
    let initial = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let source = named_port(&initial, "a")?;
    let target = named_port(&initial, "b")?;
    let armed = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(source.center, true),
    )?;
    assert!(armed.intents.is_empty());
    assert!(armed.connection_events.is_empty());
    assert_eq!(host.allocations, 0);
    let dragging = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(target.center)],
    )?;
    assert!(dragging.errors.is_empty());
    assert_eq!(host.allocations, 1);
    assert_eq!(
        dragging
            .connection_events
            .iter()
            .filter(|event| matches!(event, ConnectionEvent::Begin(_)))
            .count(),
        1
    );
    assert_eq!(
        dragging
            .connection_preview
            .as_ref()
            .ok_or("drag preview")?
            .state,
        ConnectionPreviewState::Valid
    );
    assert_eq!(session.document().raw_json(), original);
    let released = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(target.center, false),
    )?;
    assert!(released.errors.is_empty());
    assert!(released.consumed_input.pointer && released.consumed_input.keyboard);
    assert_eq!(
        released
            .connection_events
            .iter()
            .filter(|event| matches!(
                event,
                ConnectionEvent::End {
                    outcome: ConnectionEndOutcome::Submitted,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(view.connection_pending());
    let request = transaction(released)?;
    let pending = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    assert!(pending.intents.is_empty());
    assert!(pending.connection_events.is_empty());
    assert_eq!(
        pending
            .connection_preview
            .as_ref()
            .ok_or("pending preview")?
            .state,
        ConnectionPreviewState::Pending(ConnectionSubmissionState::Awaiting)
    );
    assert_eq!(session.document().raw_json(), original);
    assert!(view.acknowledge_connection(&session).is_err());
    let prepared = session.prepare(request, &StandardDialect)?;
    view.bind_connection_prepared(&prepared)?;
    assert!(view.acknowledge_connection(&session).is_err());
    session.apply(prepared)?;
    view.acknowledge_connection(&session)?;
    assert!(!view.connection_pending());
    let edge = session.document().edge("created-1").ok_or("created edge")?;
    assert_eq!(edge.endpoint(ConnectorEndpointKind::From)?, source.endpoint);
    assert_eq!(edge.endpoint(ConnectorEndpointKind::To)?, target.endpoint);
    assert_eq!(session.history_usage().undo_steps, 1);
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), original);
    Ok(())
}

#[test]
fn zoom_uses_one_frame_transform_and_target_start_hits_the_reported_ports() -> TestResult {
    let mut session = session(false)?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host::default();
    let initial = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let zoomed = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![
            Event::PointerMoved(initial.viewport.center()),
            Event::Zoom(1.5),
        ],
    )?;
    assert!(zoomed.errors.is_empty());
    assert_eq!(zoomed.frame_camera, initial.frame_camera);
    assert!(view.camera().zoom() > zoomed.frame_camera.zoom());
    let settled = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    assert_eq!(settled.frame_camera, view.camera());
    for output in [&zoomed, &settled] {
        for port in &output.ports {
            let local = output.frame_camera.world_to_view(
                port.world,
                WorldPoint::new(
                    f64::from(output.viewport.width()),
                    f64::from(output.viewport.height()),
                )?,
            );
            assert!((f64::from(port.center.x - output.viewport.left()) - local.x).abs() < 0.001);
            assert!((f64::from(port.center.y - output.viewport.top()) - local.y).abs() < 0.001);
        }
    }
    // Space plus a press on a visible port still belongs to board navigation.
    let pan_origin = named_port(&settled, "a")?.center;
    frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(pan_origin)],
    )?;
    let camera_before_pan = view.camera();
    let mut pan_start = vec![key(Key::Space)];
    pan_start.extend(pointer(pan_origin, true));
    let pressed = frame(&ctx, &mut view, &session, &mut host, pan_start)?;
    let pan_target = pan_origin + Vec2::new(40., 12.);
    let panned = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(pan_target)],
    )?;
    let mut pan_end = pointer(pan_target, false);
    pan_end.push(Event::Key {
        key: Key::Space,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    let ended = frame(&ctx, &mut view, &session, &mut host, pan_end)?;
    for output in [&pressed, &panned, &ended] {
        assert!(output.errors.is_empty());
        assert!(output.intents.is_empty());
        assert!(output.connection_events.is_empty());
        assert!(output.connection_preview.is_none());
    }
    assert_ne!(view.camera(), camera_before_pan);
    assert_eq!(host.allocations, 0);
    assert!(session.document().edges().is_empty());
    let settled = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let batch_origin = named_port(&settled, "a")?.center;
    let batch_target = named_port(&settled, "b")?.center;
    let batch_camera = view.camera();
    let mut batch = vec![key(Key::Space)];
    batch.extend(pointer(batch_origin, true));
    batch.extend(pointer(batch_target, false));
    batch.push(Event::Key {
        key: Key::Space,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    let ended = frame(&ctx, &mut view, &session, &mut host, batch)?;
    assert!(ended.intents.is_empty());
    assert!(ended.connection_events.is_empty());
    assert_eq!(host.allocations, 0);
    view.set_camera(batch_camera);
    let settled = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let source = named_port(&settled, "a")?;
    let target = named_port(&settled, "b")?;
    frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(target.center, true),
    )?;
    let dragging = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(source.center)],
    )?;
    assert!(dragging.connection_events.iter().any(|event| matches!(event, ConnectionEvent::Begin(gesture) if gesture.start == ConnectorEndpointKind::To)));
    let released = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(source.center, false),
    )?;
    let prepared = session.prepare(transaction(released)?, &StandardDialect)?;
    view.bind_connection_prepared(&prepared)?;
    session.apply(prepared)?;
    view.acknowledge_connection(&session)?;
    let edge = session
        .document()
        .edge("created-1")
        .ok_or("created target-start edge")?;
    assert_eq!(edge.endpoint(ConnectorEndpointKind::From)?, source.endpoint);
    assert_eq!(edge.endpoint(ConnectorEndpointKind::To)?, target.endpoint);
    assert_eq!(
        session
            .document()
            .node("a")
            .ok_or("source node")?
            .bounds()
            .x,
        -300.
    );
    Ok(())
}

#[test]
fn terminal_escape_consumes_release_and_competing_board_commands() -> TestResult {
    let session = session(false)?;
    let original = session.document().raw_json().to_owned();
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    view.set_selection(Selection {
        nodes: BTreeSet::from(["a".into()]),
        edges: BTreeSet::new(),
    });
    let mut host = Host::default();
    let initial = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let source = named_port(&initial, "a")?;
    let target = named_port(&initial, "b")?;
    frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(source.center, true),
    )?;
    let dragging = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(target.center)],
    )?;
    assert!(
        dragging
            .connection_events
            .iter()
            .any(|event| matches!(event, ConnectionEvent::Begin(_)))
    );
    let mut events = vec![key(Key::Escape)];
    events.extend(pointer(target.center, false));
    events.extend([key(Key::Delete), key(Key::Enter)]);
    let cancelled = frame(&ctx, &mut view, &session, &mut host, events)?;
    assert!(cancelled.errors.is_empty());
    assert!(cancelled.intents.is_empty());
    assert!(cancelled.consumed_input.pointer && cancelled.consumed_input.keyboard);
    assert_eq!(
        cancelled
            .connection_events
            .iter()
            .filter(|event| matches!(
                event,
                ConnectionEvent::Cancel {
                    reason: ConnectionCancelReason::Escape,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(
        !cancelled
            .connection_events
            .iter()
            .any(|event| matches!(event, ConnectionEvent::End { .. }))
    );
    assert!(!view.has_gesture());
    assert_eq!(session.document().raw_json(), original);
    assert_eq!(session.history_usage().undo_steps, 0);
    let idle = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    assert!(idle.intents.is_empty());
    assert!(idle.connection_events.is_empty());
    Ok(())
}

#[test]
fn embedded_and_overlay_ownership_prevent_capture_and_takeover_cancels() -> TestResult {
    let session = session(false)?;
    let original = session.document().raw_json().to_owned();
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host::default();
    let initial = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let source = named_port(&initial, "a")?;
    let target = named_port(&initial, "b")?;
    for embedded in [true, false] {
        host.body_capture = embedded;
        host.exclusions = if embedded {
            vec![]
        } else {
            vec![source.hit_rect.expand(10.)]
        };
        let pressed = frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            pointer(source.center, true),
        )?;
        assert!(!view.has_gesture());
        assert!(pressed.intents.is_empty());
        let released = frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            pointer(source.center, false),
        )?;
        assert!(released.intents.is_empty());
        assert!(released.connection_events.is_empty());
        assert_eq!(host.allocations, 0);
    }
    host.exclusions.clear();
    frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(source.center, true),
    )?;
    let dragging = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(target.center)],
    )?;
    assert!(
        dragging
            .connection_events
            .iter()
            .any(|event| matches!(event, ConnectionEvent::Begin(_)))
    );
    host.ownership.ime = true;
    let taken = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        pointer(target.center, false),
    )?;
    assert!(taken.intents.is_empty());
    assert!(taken.consumed_input.pointer && taken.consumed_input.ime);
    assert!(taken.connection_events.iter().any(|event| matches!(
        event,
        ConnectionEvent::Cancel {
            reason: ConnectionCancelReason::HostTakeover,
            ..
        }
    )));
    assert!(!view.has_gesture());
    assert_eq!(session.document().raw_json(), original);
    Ok(())
}

#[test]
fn selected_legacy_reconnect_restores_exact_raw_binding_without_history() -> TestResult {
    let session = session(true)?;
    let original = session.document().raw_json().to_owned();
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    view.set_selection(Selection {
        nodes: BTreeSet::new(),
        edges: BTreeSet::from(["legacy".into()]),
    });
    let mut host = Host::default();
    let initial = frame(&ctx, &mut view, &session, &mut host, vec![])?;
    let handle = initial
        .reconnect_handles
        .iter()
        .find(|handle| handle.end == ConnectorEndpointKind::From)
        .ok_or("source reconnect handle")?;
    assert_eq!(handle.original.binding, PortBinding::Legacy(None));
    assert_eq!(
        handle.display_endpoint.binding,
        PortBinding::Legacy(Some(Side::Top))
    );
    // Four points crosses the connection threshold but remains within egui's click
    // tolerance. The second terminal release is an actual double click on the card.
    for _ in 0..2 {
        frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            pointer(handle.center, true),
        )?;
        let dragging = frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            vec![Event::PointerMoved(handle.center - Vec2::new(0., 4.))],
        )?;
        assert!(dragging.connection_preview.is_some());
        assert_eq!(
            dragging.edges.len(),
            1,
            "the original canonical edge remains visible"
        );
        let released = frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            pointer(handle.center, false),
        )?;
        assert!(released.errors.is_empty());
        assert!(released.intents.is_empty());
        assert!(released.consumed_input.pointer);
        assert_eq!(
            released
                .connection_events
                .iter()
                .filter(|event| matches!(
                    event,
                    ConnectionEvent::End {
                        outcome: ConnectionEndOutcome::NoOp,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(!view.has_gesture());
    }
    assert!(ctx.input(|input| input.pointer.button_double_clicked(PointerButton::Primary)));
    assert_eq!(host.allocations, 0);
    assert_eq!(session.document().raw_json(), original);
    assert_eq!(session.history_usage().undo_steps, 0);
    Ok(())
}
