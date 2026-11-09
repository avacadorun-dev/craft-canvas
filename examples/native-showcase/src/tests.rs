//! Actual showcase-host RawInput regressions; these do not establish OS/GPU acceptance.
use super::*;
use craft_canvas_egui::{CanvasViewOutput, ConnectionEndOutcome, ConnectionEvent};
use craft_canvas_model::PortBinding;
use egui::{Event, Modifiers, PointerButton, RawInput};

type TestResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Harness {
    ctx: egui::Context,
    view: CanvasView,
    session: CanvasSession,
    host: Host,
    time: f64,
}
impl Harness {
    fn new() -> TestResult<Self> {
        let limits = CanvasLimits::default();
        let ctx = egui::Context::default();
        ctx.style_mut(|style| style.animation_time = 0.0);
        Ok(Self {
            ctx,
            view: CanvasView::default(),
            session: CanvasSession::new(CanvasDocument::parse(FIXTURE, &limits)?, limits)?,
            host: Host::default(),
            time: 0.0,
        })
    }
    fn frame(&mut self, events: Vec<Event>) -> TestResult<CanvasViewOutput> {
        self.time += 1.0 / 60.0;
        let ctx = self.ctx.clone();
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 900.0))),
            time: Some(self.time),
            focused: true,
            events,
            ..Default::default()
        };
        self.host.prepare_input(&ctx, &input);
        self.host.ownership = CanvasInputOwnership::default();
        let mut output = None;
        let mut rendered = ctx.run_ui(input, |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut config = CanvasViewConfig::new(Id::new("showcase-host-regression"), ui);
                config.connections = Some(ConnectionOptions::default());
                output = Some(self.view.show(
                    ui,
                    &self.session,
                    &StandardDialect,
                    &mut self.host,
                    &config,
                ));
            });
        });
        rendered.textures_delta.clear();
        rendered.platform_output.commands.clear();
        let output = output.ok_or("actual showcase view did not run")?;
        assert!(output.errors.is_empty(), "{:?}", output.errors);
        Ok(output)
    }
    fn settled(&mut self) -> TestResult<CanvasViewOutput> {
        self.frame(vec![])?;
        self.frame(vec![])?;
        self.frame(vec![])
    }
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
fn named_port(output: &CanvasViewOutput, node: &str, port: &str) -> TestResult<Pos2> {
    output
        .ports
        .iter()
        .find(|visible| {
            visible.endpoint.node_id == node
                && matches!(&visible.endpoint.binding, PortBinding::Named(key) if key.id == port)
        })
        .map(|visible| visible.center)
        .ok_or_else(|| format!("missing actual port {node}/{port}").into())
}

#[test]
fn a_connection_started_outside_can_cross_the_scene_body_without_host_takeover() -> TestResult {
    let mut harness = Harness::new()?;
    let initial = harness.settled()?;
    let from = named_port(&initial, "a", "main")?;
    let to = named_port(&initial, "b", "input")?;
    let scene = initial
        .nodes
        .iter()
        .find(|node| node.node_id == "b")
        .ok_or("scene body")?
        .body_clip
        .center();
    harness.frame(pointer(from, true))?;
    let begin = harness.frame(vec![Event::PointerMoved(from + Vec2::new(30.0, 0.0))])?;
    assert!(
        begin
            .connection_events
            .iter()
            .any(|event| matches!(event, ConnectionEvent::Begin(_)))
    );

    let crossing = harness.frame(vec![Event::PointerMoved(scene)])?;
    assert!(!harness.host.input_ownership().pointer);
    assert!(harness.view.has_gesture());
    assert!(
        !crossing
            .connection_events
            .iter()
            .any(|event| matches!(event, ConnectionEvent::Cancel { .. }))
    );
    assert!(crossing.intents.is_empty());

    harness.frame(vec![Event::PointerMoved(to)])?;
    let release = harness.frame(pointer(to, false))?;
    assert!(release.connection_events.iter().any(|event| matches!(
        event,
        ConnectionEvent::End {
            outcome: ConnectionEndOutcome::Submitted,
            ..
        }
    )));
    assert_eq!(release.intents.len(), 1);
    assert!(harness.view.connection_pending());
    assert_eq!(harness.session.document().raw_json(), FIXTURE);
    assert_eq!(harness.session.history_usage().undo_steps, 0);
    Ok(())
}

#[test]
fn the_actual_scene_scrollbar_owns_press_drag_and_release_without_board_mutation() -> TestResult {
    let mut harness = Harness::new()?;
    harness.settled()?;
    let scroll_id = harness
        .host
        .scene_scroll_id
        .ok_or("actual scroll area ID")?;
    let bar = harness
        .ctx
        .read_response(scroll_id.with(1_usize))
        .ok_or("actual vertical scrollbar")?
        .rect;
    let from = Pos2::new(bar.center().x, bar.top() + 8.0);
    let to = from + Vec2::new(0.0, 45.0);
    let before = egui::scroll_area::State::load(&harness.ctx, scroll_id)
        .ok_or("initial scroll state")?
        .offset
        .y;

    for events in [
        pointer(from, true),
        vec![Event::PointerMoved(to)],
        pointer(to, false),
    ] {
        let output = harness.frame(events)?;
        assert!(
            harness.host.input_ownership().pointer,
            "the real scroll widget must retain its terminal frame"
        );
        assert!(output.connection_events.is_empty());
        assert!(output.intents.is_empty());
    }
    let after = egui::scroll_area::State::load(&harness.ctx, scroll_id)
        .ok_or("final scroll state")?
        .offset
        .y;
    assert!(
        after > before,
        "the actual scroll widget must have moved its content"
    );
    harness.frame(vec![])?;
    assert!(!harness.host.input_ownership().pointer);
    assert!(!harness.view.has_gesture());
    assert_eq!(harness.session.document().raw_json(), FIXTURE);
    assert_eq!(harness.session.history_usage().undo_steps, 0);
    Ok(())
}

struct ShowcaseHarness {
    ctx: egui::Context,
    app: Showcase,
    time: f64,
}
impl ShowcaseHarness {
    fn new(testing: bool) -> TestResult<Self> {
        let ctx = egui::Context::default();
        ctx.style_mut(|style| style.animation_time = 0.0);
        // Exercise the normal single-pass boundary: the header paints before
        // the canvas release prepares a candidate later in that same pass.
        ctx.options_mut(|options| options.max_passes = std::num::NonZeroUsize::MIN);
        let mut app = Showcase::new("header-regression".into(), None)?;
        if testing {
            app.acknowledgement = Acknowledgement::Hold;
        }
        let mut harness = Self {
            ctx,
            app,
            time: 0.0,
        };
        for _ in 0..3 {
            harness.frame(vec![])?;
        }
        if testing {
            harness.click("button:testing")?;
            for _ in 0..3 {
                harness.frame(vec![])?;
            }
        }
        Ok(harness)
    }
    fn frame(&mut self, events: Vec<Event>) -> TestResult {
        self.time += 1.0 / 60.0;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 900.0))),
            time: Some(self.time),
            focused: true,
            events,
            ..Default::default()
        };
        self.app.host.prepare_input(&self.ctx, &input);
        let mut output = self.ctx.run_ui(input, |root| self.app.show(root));
        output.textures_delta.clear();
        output.platform_output.commands.clear();
        let observation = self
            .app
            .observation
            .as_ref()
            .ok_or("showcase observation")?;
        let errors = observation.state["engine_errors"]
            .as_array()
            .ok_or("engine errors")?;
        assert!(errors.is_empty(), "{errors:?}");
        Ok(())
    }
    fn target(&self, key: &str) -> TestResult<Pos2> {
        self.app
            .observation
            .as_ref()
            .and_then(|observation| observation.targets.get(key))
            .copied()
            .ok_or_else(|| format!("missing interactive target {key}").into())
    }
    fn click(&mut self, key: &str) -> TestResult {
        let point = self.target(key)?;
        self.frame(pointer(point, true))?;
        self.frame(pointer(point, false))
    }
    fn connect(&mut self) -> TestResult {
        let from = self.target("port:a:source:main")?;
        let to = self.target("port:b:target:input")?;
        self.frame(pointer(from, true))?;
        self.frame(vec![Event::PointerMoved(from.lerp(to, 0.5))])?;
        self.frame(vec![Event::PointerMoved(to)])?;
        self.frame(pointer(to, false))
    }
}

#[test]
fn held_connection_reject_uses_an_enabled_header_target_without_applying_or_undoing() -> TestResult
{
    let mut harness = ShowcaseHarness::new(true)?;
    harness.connect()?;
    assert!(matches!(
        harness.app.view.connection_status(),
        Some(ConnectionSubmissionState::Awaiting)
    ));
    assert!(harness.app.prepared.is_some());
    // The held candidate exists after this frame, but Reject was painted disabled.
    // A driver must wait for the next actual header instead of pressing its rect.
    assert!(harness.target("button:reject").is_err());
    assert!(harness.target("button:apply").is_err());
    harness.frame(vec![])?;
    harness.click("button:reject")?;
    assert!(matches!(
        harness.app.view.connection_status(),
        Some(ConnectionSubmissionState::Rejected(_))
    ));
    assert!(harness.app.prepared.is_none());
    assert_eq!(harness.app.session.identity().revision(), 0);
    assert_eq!(harness.app.session.document().raw_json(), FIXTURE);
    assert_eq!(harness.app.session.history_usage().undo_steps, 0);
    assert_eq!(harness.app.session.history_usage().redo_steps, 0);
    assert_eq!(harness.app.counters.acknowledgements, 0);
    assert_eq!(harness.app.counters.submissions, 1);
    assert_eq!(harness.app.counters.intents, 1);
    Ok(())
}

#[test]
fn discard_conflicted_connection_preserves_history_and_allows_another_direct_drag() -> TestResult {
    let mut harness = ShowcaseHarness::new(true)?;
    harness.connect()?;
    harness.frame(vec![])?;
    harness.click("button:external")?;
    assert!(matches!(
        harness.app.view.connection_status(),
        Some(ConnectionSubmissionState::Conflict)
    ));
    let canonical = harness.app.session.document().raw_json().to_owned();
    let identity = harness.app.session.identity();
    let undo = harness.app.session.history_usage().undo_steps;
    let redo = harness.app.session.history_usage().redo_steps;

    // Recovery remains visible when the technical controls are collapsed.
    harness.click("button:testing")?;
    harness.frame(vec![])?;
    assert!(harness.target("button:external").is_err());
    harness.click("button:discard")?;
    assert!(!harness.app.view.connection_pending());
    assert!(harness.app.prepared.is_none());
    assert_eq!(harness.app.session.document().raw_json(), canonical);
    assert_eq!(harness.app.session.identity(), identity);
    assert_eq!(harness.app.session.history_usage().undo_steps, undo);
    assert_eq!(harness.app.session.history_usage().redo_steps, redo);
    assert_eq!(harness.app.counters.acknowledgements, 0);

    harness.app.acknowledgement = Acknowledgement::Immediate;
    harness.frame(vec![])?;
    harness.connect()?;
    assert!(!harness.app.view.connection_pending());
    assert!(
        harness
            .app
            .session
            .document()
            .edge("showcase-edge-2")
            .is_some()
    );
    assert_eq!(
        harness.app.session.identity().revision(),
        identity.revision() + 1
    );
    assert_eq!(harness.app.session.history_usage().undo_steps, undo + 1);
    assert_eq!(harness.app.counters.acknowledgements, 1);
    Ok(())
}

#[test]
fn ordinary_showcase_connects_directly_and_undoes_with_testing_controls_collapsed() -> TestResult {
    let mut harness = ShowcaseHarness::new(false)?;
    assert!(matches!(
        harness.app.acknowledgement,
        Acknowledgement::Immediate
    ));
    harness.connect()?;
    assert!(!harness.app.view.connection_pending());
    assert_eq!(harness.app.session.document().edges().len(), 2);
    assert_eq!(harness.app.counters.acknowledgements, 1);
    harness.frame(vec![])?;
    assert!(harness.target("button:external").is_err());
    assert!(harness.target("button:reject").is_err());
    harness.click("button:undo")?;
    assert_eq!(harness.app.session.document().raw_json(), FIXTURE);
    assert_eq!(harness.app.session.history_usage().undo_steps, 0);
    assert_eq!(harness.app.session.history_usage().redo_steps, 1);
    Ok(())
}

#[test]
fn capture_input_trace_redacts_text_and_character_key_payloads() {
    for event in [
        Event::Text("private synthetic text".into()),
        Event::Paste("private synthetic clipboard".into()),
        Event::Ime(egui::ImeEvent::Commit(
            "private synthetic composition".into(),
        )),
    ] {
        let trace = capture::input_event_json(&event);
        assert_eq!(trace.as_object().map(|object| object.len()), Some(1));
        assert!(!trace.to_string().contains("private"));
    }
    let key = capture::input_event_json(&Event::Key {
        key: egui::Key::A,
        physical_key: Some(egui::Key::B),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    assert_eq!(key["control"], "redacted");
    assert!(key.get("physical_key").is_none());
}
