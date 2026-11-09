use super::*;
use crate::{NodeBodyResponse, port_geometry::PortFrameConfig};
use craft_canvas_model::{CanvasDocument, CanvasLimits, NodeRecord, StandardDialect, WorkControl};
use egui::{Modifiers, Vec2};

const BOARD: &str = r#"{"nodes":[
 {"id":"a","type":"text","text":"A","x":-160,"y":-40,"width":80,"height":80,"craftCanvas":{"version":1,"ports":[{"id":"out","kind":"source","side":"right","x":80,"y":40}]}},
 {"id":"b","type":"text","text":"B","x":80,"y":-40,"width":80,"height":80,"craftCanvas":{"version":1,"ports":[{"id":"in","kind":"target","side":"left","x":0,"y":40}]}},
 {"id":"c","type":"text","text":"C","x":80,"y":120,"width":80,"height":80}],
 "edges":[{"id":"old","fromNode":"a","toNode":"b","toSide":"","label":"keep","unknown":{"precise":12345678901234567890}}]}"#;

#[derive(Default)]
struct Host {
    allocations: usize,
}
impl CanvasViewHost for Host {
    fn paint_body(
        &mut self,
        _ui: &mut egui::Ui,
        _node: &NodeRecord,
        _rect: Rect,
        _zoom: f64,
    ) -> NodeBodyResponse {
        NodeBodyResponse::default()
    }
    fn allocate_edge_id(
        &mut self,
        _base: ContentIdentity,
        _origin: &PortEndpoint,
    ) -> Result<Option<String>> {
        self.allocations += 1;
        Ok(Some(format!("created-{}", self.allocations)))
    }
}
struct Fixture {
    session: CanvasSession,
    controller: ConnectionController,
    host: Host,
    frame: PortFrame,
    options: ConnectionOptions,
    handles: Vec<VisibleReconnectHandle>,
}
impl Fixture {
    fn new() -> Result<Self> {
        Self::from_raw(BOARD)
    }
    fn from_raw(raw: &str) -> Result<Self> {
        let limits = CanvasLimits::default();
        let session = CanvasSession::new(CanvasDocument::parse(raw, &limits)?, limits)?;
        let host = Host::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 500.0));
        let config = PortFrameConfig {
            camera: Camera::default(),
            viewport,
            clip: viewport,
            port_radius: 5.0,
            reconnect_radius: 8.0,
        };
        let frame = PortFrame::build(
            &session,
            &StandardDialect,
            &host,
            config,
            NodeRecord::bounds,
            &mut WorkControl::new(session.limits().max_work_units, None),
        )?;
        Ok(Self {
            session,
            controller: ConnectionController::default(),
            host,
            frame,
            options: ConnectionOptions::default(),
            handles: vec![],
        })
    }
    fn point(&self, node: &str, binding: PortBinding) -> Result<Pos2> {
        self.frame
            .ports
            .iter()
            .find(|port| port.endpoint.node_id == node && port.endpoint.binding == binding)
            .map(|port| port.center)
            .ok_or_else(|| CanvasError::NotFound(node.into()))
    }
    fn source(&self) -> Result<Pos2> {
        self.named("a", "out", PortKind::Source)
    }
    fn target(&self) -> Result<Pos2> {
        self.named("b", "in", PortKind::Target)
    }
    fn named(&self, node: &str, id: &str, kind: PortKind) -> Result<Pos2> {
        self.point(
            node,
            PortBinding::Named(craft_canvas_model::PortKey {
                id: id.into(),
                kind,
            }),
        )
    }
    fn run(
        &mut self,
        events: &[Event],
        pointer: Option<Pos2>,
        down: bool,
        ownership: CanvasInputOwnership,
    ) -> ConnectionStep {
        self.run_with_exclusions(events, pointer, down, ownership, &[])
    }
    fn run_with_exclusions(
        &mut self,
        events: &[Event],
        pointer: Option<Pos2>,
        down: bool,
        ownership: CanvasInputOwnership,
        exclusions: &[Rect],
    ) -> ConnectionStep {
        self.controller.step(ConnectionInput {
            session: &self.session,
            dialect: &StandardDialect,
            host: &mut self.host,
            ports: &self.frame,
            handles: &self.handles,
            options: Some(&self.options),
            board_id: Id::new("test-board"),
            viewport_id: ViewportId::ROOT,
            viewport: self.frame.config.viewport,
            camera: self.frame.config.camera,
            events,
            pointer_pos: pointer,
            primary_down: down,
            focused: true,
            enabled: true,
            ownership,
            exclusions,
        })
    }
    fn submit(&mut self) -> Result<CanvasTransaction> {
        let source = self.source()?;
        let target = self.target()?;
        self.run(
            &[
                mouse(source, true),
                Event::PointerMoved(target),
                mouse(target, false),
            ],
            Some(target),
            false,
            CanvasInputOwnership::default(),
        )
        .transaction
        .ok_or_else(|| CanvasError::Invalid("test connection did not submit".into()))
    }
}
fn mouse(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn touch(id: u64, phase: TouchPhase, pos: Pos2) -> Event {
    Event::Touch {
        device_id: TouchDeviceId(7),
        id: TouchId(id),
        phase,
        pos,
        force: None,
    }
}
fn terminal_count(events: &[ConnectionEvent]) -> usize {
    events
        .iter()
        .filter(|event| {
            matches!(
                event,
                ConnectionEvent::End { .. } | ConnectionEvent::Cancel { .. }
            )
        })
        .count()
}

#[test]
fn touch_release_order_distinguishes_normal_pointer_departure_from_lost_capture() -> Result<()> {
    for gone_first in [false, true] {
        let mut fixture = Fixture::new()?;
        let source = fixture.source()?;
        let target = fixture.target()?;
        let begin = fixture.run(
            &[
                touch(1, TouchPhase::Start, source),
                touch(1, TouchPhase::Move, target),
            ],
            Some(target),
            true,
            CanvasInputOwnership::default(),
        );
        assert!(matches!(
            begin.events.first(),
            Some(ConnectionEvent::Begin(_))
        ));
        let events = if gone_first {
            vec![Event::PointerGone, touch(1, TouchPhase::End, target)]
        } else {
            vec![touch(1, TouchPhase::End, target), Event::PointerGone]
        };
        let output = fixture.run(&events, None, false, CanvasInputOwnership::default());
        assert_eq!(output.transaction.is_some(), !gone_first);
        assert_eq!(terminal_count(&output.events), 1);
        assert!(output.blocking && output.consumed_input.pointer && output.consumed_input.touch);
        if gone_first {
            assert!(matches!(
                output.events.last(),
                Some(ConnectionEvent::Cancel {
                    reason: ConnectionCancelReason::PointerLost,
                    ..
                })
            ));
        }
    }
    Ok(())
}

#[test]
fn a_foreign_contact_and_synthesized_mouse_release_cannot_finish_the_owned_touch() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let source = fixture.source()?;
    let target = fixture.target()?;
    fixture.run(
        &[
            mouse(source, true),
            touch(1, TouchPhase::Start, source),
            touch(1, TouchPhase::Move, target),
        ],
        Some(target),
        true,
        CanvasInputOwnership::default(),
    );
    let foreign = fixture.run(
        &[touch(2, TouchPhase::End, target), mouse(target, false)],
        Some(target),
        false,
        CanvasInputOwnership::default(),
    );
    assert!(foreign.transaction.is_none() && fixture.controller.has_gesture());
    let completed = fixture.run(
        &[
            mouse(target, false),
            touch(1, TouchPhase::End, target),
            Event::PointerGone,
        ],
        None,
        false,
        CanvasInputOwnership::default(),
    );
    assert!(completed.transaction.is_some());
    assert_eq!(fixture.host.allocations, 1);
    let next = fixture.run(
        &[mouse(target, false)],
        Some(target),
        false,
        CanvasInputOwnership::default(),
    );
    assert!(next.transaction.is_none() && next.blocking);
    Ok(())
}

#[test]
fn threshold_click_and_terminal_host_takeover_never_leak_into_a_second_gesture() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let source = fixture.source()?;
    let target = fixture.target()?;
    let click = fixture.run(
        &[mouse(source, true), mouse(source, false)],
        Some(source),
        false,
        CanvasInputOwnership::default(),
    );
    assert!(click.blocking && click.events.is_empty() && click.transaction.is_none());
    assert_eq!(fixture.host.allocations, 0);
    fixture.run(
        &[mouse(source, true), Event::PointerMoved(target)],
        Some(target),
        true,
        CanvasInputOwnership::default(),
    );
    let cancelled = fixture.run(
        &[mouse(target, false), mouse(source, true)],
        Some(source),
        true,
        CanvasInputOwnership {
            keyboard: true,
            ..CanvasInputOwnership::default()
        },
    );
    assert!(matches!(
        cancelled.events.as_slice(),
        [ConnectionEvent::Cancel {
            reason: ConnectionCancelReason::HostTakeover,
            ..
        }]
    ));
    assert!(cancelled.blocking && !fixture.controller.has_gesture());
    assert_eq!(fixture.host.allocations, 1);
    Ok(())
}

#[test]
fn public_cancellation_queues_one_terminal_and_consumes_the_next_release() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let source = fixture.source()?;
    let target = fixture.target()?;
    fixture.run(
        &[mouse(source, true), Event::PointerMoved(target)],
        Some(target),
        true,
        CanvasInputOwnership::default(),
    );
    fixture
        .controller
        .cancel(ConnectionCancelReason::CameraChanged);
    fixture
        .controller
        .cancel(ConnectionCancelReason::HostCancelled);
    let output = fixture.run(
        &[mouse(target, false), mouse(source, true)],
        Some(source),
        true,
        CanvasInputOwnership::default(),
    );
    assert!(matches!(
        output.events.as_slice(),
        [ConnectionEvent::Cancel {
            reason: ConnectionCancelReason::CameraChanged,
            ..
        }]
    ));
    assert!(output.blocking && output.transaction.is_none() && !fixture.controller.has_gesture());
    Ok(())
}

#[test]
fn target_start_normalizes_roles_and_retains_actual_initiation() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let source = fixture.source()?;
    let target = fixture.target()?;
    let output = fixture.run(
        &[
            mouse(target, true),
            Event::PointerMoved(source),
            mouse(source, false),
        ],
        Some(source),
        false,
        CanvasInputOwnership::default(),
    );
    let transaction = output
        .transaction
        .ok_or_else(|| CanvasError::Invalid("target-start did not submit".into()))?;
    assert!(matches!(
        transaction.commands(),
        [CanvasCommand::ConnectPorts {
            start: ConnectorEndpointKind::To,
            ..
        }]
    ));
    let prepared = fixture.session.prepare(transaction, &StandardDialect)?;
    let edge = prepared
        .document()
        .edge("created-1")
        .ok_or_else(|| CanvasError::NotFound("created-1".into()))?;
    assert_eq!(edge.endpoint(ConnectorEndpointKind::From)?.node_id, "a");
    assert_eq!(edge.endpoint(ConnectorEndpointKind::To)?.node_id, "b");
    Ok(())
}

#[test]
fn both_legacy_ends_restore_omitted_or_empty_bindings_without_history() -> Result<()> {
    for colocated in [false, true] {
        // Named handles can occupy the omitted-side Top fallback of a legacy edge.
        let raw = if colocated {
            BOARD
                .replace("\"x\":80,\"y\":40", "\"x\":40,\"y\":0")
                .replace("\"x\":0,\"y\":40", "\"x\":40,\"y\":0")
        } else {
            BOARD.to_owned()
        };
        let mut fixture = Fixture::from_raw(&raw)?;
        let edge = fixture
            .session
            .document()
            .edge("old")
            .ok_or_else(|| CanvasError::NotFound("old".into()))?;
        fixture.handles = fixture.frame.reconnect_handles(
            edge,
            &StandardDialect,
            &mut WorkControl::new(1000, None),
        )?;
        let points: Vec<_> = fixture.handles.iter().map(|handle| handle.center).collect();
        assert_eq!(points.len(), 2);
        for point in points {
            let output = fixture.run(
                &[
                    mouse(point, true),
                    Event::PointerMoved(point - Vec2::new(0.0, 40.0)),
                    mouse(point, false),
                ],
                Some(point),
                false,
                CanvasInputOwnership::default(),
            );
            assert!(matches!(
                output.events.last(),
                Some(ConnectionEvent::End {
                    outcome: ConnectionEndOutcome::NoOp,
                    ..
                })
            ));
            assert!(output.transaction.is_none());
        }
        assert_eq!(fixture.session.document().raw_json(), raw);
        assert_eq!(fixture.session.history_usage().undo_steps, 0);
    }
    Ok(())
}

#[test]
fn rejected_submission_retries_only_explicitly_and_exact_receipt_acknowledges_once() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let transaction = fixture.submit()?;
    let prepared = fixture.session.prepare(transaction, &StandardDialect)?;
    fixture.controller.bind_connection_prepared(&prepared)?;
    assert!(fixture.controller.dismiss_connection().is_err());
    fixture
        .controller
        .reject_connection("synthetic write refusal")?;
    let retry = fixture
        .controller
        .retry_connection(&fixture.session, &StandardDialect)?;
    assert!(
        fixture
            .controller
            .retry_connection(&fixture.session, &StandardDialect)
            .is_err()
    );
    let retry_prepared = fixture.session.prepare(retry, &StandardDialect)?;
    fixture
        .controller
        .bind_connection_prepared(&retry_prepared)?;
    fixture.session.apply(retry_prepared)?;
    fixture
        .controller
        .acknowledge_connection(&fixture.session)?;
    assert!(!fixture.controller.connection_pending());
    assert!(
        fixture
            .controller
            .acknowledge_connection(&fixture.session)
            .is_err()
    );
    assert_eq!(fixture.session.history_usage().undo_steps, 1);
    Ok(())
}

#[test]
fn acknowledgement_refuses_unbound_foreign_and_unrelated_next_revision() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let transaction = fixture.submit()?;
    assert!(
        fixture
            .controller
            .acknowledge_connection(&fixture.session)
            .is_err()
    );
    let prepared = fixture.session.prepare(transaction, &StandardDialect)?;
    fixture.controller.bind_connection_prepared(&prepared)?;
    let foreign = Fixture::new()?;
    assert!(
        fixture
            .controller
            .acknowledge_connection(&foreign.session)
            .is_err()
    );
    let unrelated = CanvasTransaction::new(
        fixture.session.identity(),
        vec![CanvasCommand::EditText {
            node_id: "a".into(),
            expected: "A".into(),
            text: "unrelated".into(),
        }],
    );
    fixture
        .session
        .apply(fixture.session.prepare(unrelated, &StandardDialect)?)?;
    assert!(
        fixture
            .controller
            .acknowledge_connection(&fixture.session)
            .is_err()
    );
    let output = fixture.run(&[], None, false, CanvasInputOwnership::default());
    assert!(output.blocking && output.transaction.is_none());
    assert_eq!(
        fixture.controller.connection_status(),
        Some(&ConnectionSubmissionState::Conflict)
    );
    assert!(fixture.controller.dismiss_connection().is_err());
    fixture
        .controller
        .reject_connection("host cancelled stale request")?;
    assert!(matches!(
        fixture
            .controller
            .retry_connection(&fixture.session, &StandardDialect),
        Err(CanvasError::StaleBase)
    ));
    fixture.controller.dismiss_connection()?;
    Ok(())
}

#[test]
fn touch_only_input_completes_without_synthesized_mouse_position_or_buttons() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let source = fixture.source()?;
    let target = fixture.target()?;
    let raw = egui::RawInput {
        events: vec![touch(1, TouchPhase::Start, source)],
        ..egui::RawInput::default()
    };
    let armed = fixture.run(&raw.events, None, false, CanvasInputOwnership::default());
    assert!(armed.blocking && fixture.controller.has_gesture());
    assert!(armed.events.is_empty());
    let raw = egui::RawInput {
        events: vec![touch(1, TouchPhase::Move, target)],
        ..egui::RawInput::default()
    };
    let dragging = fixture.run(&raw.events, None, false, CanvasInputOwnership::default());
    assert!(matches!(
        dragging.events.first(),
        Some(ConnectionEvent::Begin(_))
    ));
    assert!(fixture.controller.has_gesture());
    let raw = egui::RawInput {
        events: vec![touch(1, TouchPhase::End, target)],
        ..egui::RawInput::default()
    };
    let completed = fixture.run(&raw.events, None, false, CanvasInputOwnership::default());
    assert!(matches!(
        completed.events.last(),
        Some(ConnectionEvent::End {
            outcome: ConnectionEndOutcome::Submitted,
            ..
        })
    ));
    assert!(completed.transaction.is_some() && completed.consumed_input.touch);
    assert_eq!(terminal_count(&completed.events), 1);
    assert_eq!(fixture.host.allocations, 1);
    Ok(())
}

#[test]
fn a_stationary_pointer_is_cancelled_when_an_exclusion_overlay_appears() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let source = fixture.source()?;
    let target = fixture.target()?;
    fixture.run(
        &[mouse(source, true), Event::PointerMoved(target)],
        Some(target),
        true,
        CanvasInputOwnership::default(),
    );
    let output = fixture.run_with_exclusions(
        &[],
        Some(target),
        true,
        CanvasInputOwnership::default(),
        &[Rect::from_center_size(target, Vec2::splat(30.0))],
    );
    assert!(matches!(
        output.events.as_slice(),
        [ConnectionEvent::Cancel {
            reason: ConnectionCancelReason::HostTakeover,
            ..
        }]
    ));
    assert!(output.blocking && output.consumed_input.pointer);
    assert!(output.transaction.is_none() && !fixture.controller.has_gesture());
    Ok(())
}

#[test]
fn coincident_self_loop_endpoints_have_separate_reachable_no_op_grips() -> Result<()> {
    for bindings in [
        "\"toNode\":\"a\"",
        "\"toNode\":\"a\",\"fromSide\":\"top\",\"toSide\":\"top\"",
    ] {
        let raw = BOARD.replace("\"toNode\":\"b\",\"toSide\":\"\"", bindings);
        let mut fixture = Fixture::from_raw(&raw)?;
        let edge = fixture
            .session
            .document()
            .edge("old")
            .ok_or_else(|| CanvasError::NotFound("old".into()))?;
        fixture.handles = fixture.frame.reconnect_handles(
            edge,
            &StandardDialect,
            &mut WorkControl::new(1000, None),
        )?;
        let [from, to] = fixture.handles.as_slice() else {
            return Err(CanvasError::Invalid(
                "self-loop must expose both endpoint grips".into(),
            ));
        };
        assert_eq!(from.end, ConnectorEndpointKind::From);
        assert_eq!(to.end, ConnectorEndpointKind::To);
        assert_eq!(from.anchor_center, to.anchor_center);
        assert_ne!(from.center, to.center);
        assert!(!from.hit_rect.intersects(to.hit_rect));
        let grips = [(from.end, from.center), (to.end, to.center)];
        for (end, point) in grips {
            let output = fixture.run(
                &[
                    mouse(point, true),
                    Event::PointerMoved(point - Vec2::new(0.0, 40.0)),
                    mouse(point, false),
                ],
                Some(point),
                false,
                CanvasInputOwnership::default(),
            );
            assert!(
                matches!(output.events.first(), Some(ConnectionEvent::Begin(ConnectionGestureInfo {
                operation: ConnectionOperation::Reconnect { edge_id, endpoint }, ..
            })) if edge_id == "old" && *endpoint == end)
            );
            assert!(matches!(
                output.events.last(),
                Some(ConnectionEvent::End {
                    outcome: ConnectionEndOutcome::NoOp,
                    ..
                })
            ));
            assert!(output.transaction.is_none());
            assert_eq!(terminal_count(&output.events), 1);
        }
        assert_eq!(fixture.session.document().raw_json(), raw);
        assert_eq!(fixture.session.history_usage().undo_steps, 0);
    }
    Ok(())
}
