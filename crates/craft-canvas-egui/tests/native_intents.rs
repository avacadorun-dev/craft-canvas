//! Actual egui event paths over synthetic boards; not OS/host interaction acceptance.
use craft_canvas_egui::*;
use craft_canvas_model::*;
use egui::{Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    let raw = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":200,"height":120,"text":"A synthetic card"}],"edges":[]}"#;
    CanvasSession::new(CanvasDocument::parse(raw, &limits)?, limits)
}
struct Host {
    capture: bool,
}
impl CanvasViewHost for Host {
    fn paint_body(
        &mut self,
        _ui: &mut egui::Ui,
        _node: &NodeRecord,
        _rect: Rect,
        _zoom: f64,
    ) -> NodeBodyResponse {
        NodeBodyResponse {
            pointer_captured: self.capture,
            keyboard_captured: self.capture,
        }
    }
}
fn frame(
    ctx: &egui::Context,
    view: &mut CanvasView,
    session: &CanvasSession,
    host: &mut Host,
    events: Vec<Event>,
    focused: bool,
) -> Result<CanvasViewOutput> {
    let mut output = None;
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 680.))),
        focused,
        events,
        ..RawInput::default()
    };
    let mut frame = ctx.run_ui(input, |root| {
        egui::CentralPanel::default().show(root, |ui| {
            let config = CanvasViewConfig::new(Id::new("synthetic-board"), ui);
            output = Some(view.show(ui, session, &StandardDialect, host, &config));
        });
    });
    // This event-contract harness has no GPU. Native renderer/host acceptance is separate.
    frame.textures_delta.clear();
    frame.platform_output.commands.clear();
    output.ok_or_else(|| CanvasError::Invalid("missing synthetic native output".into()))
}
fn press(pos: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    ]
}
fn release(pos: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ]
}
#[test]
fn native_drag_is_only_a_preview_until_one_guarded_release_transaction() -> TestResult {
    let mut session = session()?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host { capture: false };
    let initial = frame(&ctx, &mut view, &session, &mut host, vec![], true)?;
    let center = initial.nodes[0].reserved.center();
    let original = session.document().raw_json().to_owned();
    let down = frame(&ctx, &mut view, &session, &mut host, press(center), true)?;
    assert!(down.intents.is_empty());
    assert!(view.has_gesture());
    let moved = center + Vec2::new(24., 12.);
    let progress = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(moved)],
        true,
    )?;
    assert!(progress.intents.is_empty());
    assert_eq!(session.document().raw_json(), original);
    let done = frame(&ctx, &mut view, &session, &mut host, release(moved), true)?;
    assert_eq!(done.intents.len(), 1);
    let transaction = done
        .intents
        .into_iter()
        .find_map(|intent| {
            if let CanvasIntent::Transaction(tx) = intent {
                Some(tx)
            } else {
                None
            }
        })
        .ok_or("transaction")?;
    let prepared = session.prepare(transaction, &StandardDialect)?;
    session.apply(prepared)?;
    assert_eq!(session.document().node("a").ok_or("a")?.bounds().x, 24.);
    assert_eq!(session.history_usage().undo_steps, 1);
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), original);
    Ok(())
}
#[test]
fn focus_loss_cancels_uncommitted_geometry_and_never_changes_canonical_data() -> TestResult {
    let session = session()?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host { capture: false };
    let first = frame(&ctx, &mut view, &session, &mut host, vec![], true)?;
    let center = first.nodes[0].reserved.center();
    frame(&ctx, &mut view, &session, &mut host, press(center), true)?;
    frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        vec![Event::PointerMoved(center + Vec2::new(30., 0.))],
        true,
    )?;
    let lost = frame(&ctx, &mut view, &session, &mut host, vec![], false)?;
    assert!(!view.has_gesture());
    assert!(lost.intents.is_empty());
    let after = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        release(center + Vec2::new(30., 0.)),
        true,
    )?;
    assert!(after.intents.is_empty());
    assert_eq!(session.document().node("a").ok_or("a")?.bounds().x, 0.);
    Ok(())
}
#[test]
fn host_media_control_capture_does_not_start_card_drag() -> TestResult {
    let session = session()?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host { capture: true };
    let first = frame(&ctx, &mut view, &session, &mut host, vec![], true)?;
    let center = first.nodes[0].reserved.center();
    let output = frame(&ctx, &mut view, &session, &mut host, press(center), true)?;
    assert!(!view.has_gesture());
    assert!(output.intents.is_empty());
    assert!(
        output.nodes[0]
            .reserved
            .contains_rect(output.nodes[0].body_clip)
    );
    Ok(())
}
#[test]
fn newer_canonical_revision_cancels_old_native_drag_instead_of_rebasing_it() -> TestResult {
    let mut session = session()?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host { capture: false };
    let first = frame(&ctx, &mut view, &session, &mut host, vec![], true)?;
    let center = first.nodes[0].reserved.center();
    frame(&ctx, &mut view, &session, &mut host, press(center), true)?;
    let change = session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::EditText {
                node_id: "a".into(),
                expected: "A synthetic card".into(),
                text: "New canonical source".into(),
            }],
        ),
        &StandardDialect,
    )?;
    session.apply(change)?;
    let output = frame(
        &ctx,
        &mut view,
        &session,
        &mut host,
        release(center + Vec2::new(30., 0.)),
        true,
    )?;
    assert!(output.intents.is_empty());
    assert!(!view.has_gesture());
    assert_eq!(session.document().node("a").ok_or("a")?.bounds().x, 0.);
    assert_eq!(
        session.document().node("a").ok_or("a")?.body(),
        "New canonical source"
    );
    Ok(())
}
#[test]
fn source_editor_retains_unicode_draft_and_exact_expected_base_for_shared_prepare() -> TestResult {
    let mut session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    *editor.text_mut() = "日本語 e\u{301} 🪶\n**Markdown**".into();
    let original = session.document().raw_json().to_owned();
    let change = session.prepare(editor.transaction(), &StandardDialect)?;
    assert_eq!(session.document().raw_json(), original);
    session.apply(change)?;
    assert_eq!(
        session.document().node("a").ok_or("a")?.body(),
        editor.text()
    );
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), original);
    assert!(matches!(
        session.prepare(editor.transaction(), &StandardDialect),
        Err(CanvasError::StaleBase)
    ));
    Ok(())
}
#[test]
fn local_geometry_at_large_world_coordinates_uses_the_same_native_card_bounds() -> TestResult {
    let session = session()?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let mut host = Host { capture: false };
    for coordinate in [-10_000_000., 10_000_000.] {
        for zoom in [Camera::MIN_ZOOM, Camera::MAX_ZOOM] {
            view.set_camera(Camera::new(
                WorldPoint {
                    x: coordinate,
                    y: coordinate,
                },
                zoom,
            )?);
            let output = frame(&ctx, &mut view, &session, &mut host, vec![], true)?;
            assert!(output.nodes.is_empty());
            assert!(output.errors.is_empty());
        }
    }
    Ok(())
}

#[derive(Default)]
struct TakeoverInput {
    host: CanvasInputOwnership,
    config: CanvasInputOwnership,
    exclusions: Vec<Rect>,
}
impl CanvasViewHost for TakeoverInput {
    fn input_ownership(&self) -> CanvasInputOwnership {
        self.host
    }
    fn paint_body(
        &mut self,
        _ui: &mut egui::Ui,
        _node: &NodeRecord,
        _rect: Rect,
        _zoom: f64,
    ) -> NodeBodyResponse {
        NodeBodyResponse::default()
    }
}
fn takeover_frame(
    ctx: &egui::Context,
    view: &mut CanvasView,
    session: &CanvasSession,
    host: &mut TakeoverInput,
    events: Vec<Event>,
) -> Result<CanvasViewOutput> {
    let mut output = None;
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 680.))),
        // Separate independent recovery clicks from egui's double-click interval.
        time: Some(ctx.input(|input| input.time) + 1.0),
        focused: true,
        events,
        ..RawInput::default()
    };
    let mut frame = ctx.run_ui(input, |root| {
        egui::CentralPanel::default().show(root, |ui| {
            let mut config = CanvasViewConfig::new(Id::new("takeover-board"), ui);
            config.input_ownership = host.config;
            config.input_exclusions.clone_from(&host.exclusions);
            output = Some(view.show(ui, session, &StandardDialect, host, &config));
        });
    });
    frame.textures_delta.clear();
    frame.platform_output.commands.clear();
    output.ok_or_else(|| CanvasError::Invalid("missing takeover output".into()))
}

#[test]
fn merged_host_ownership_terminates_geometry_instead_of_pausing_it() -> TestResult {
    let none = CanvasInputOwnership::default();
    // Exercise each merged ownership channel across the three geometry captures.
    for (kind, host_owner, config_owner) in [
        (
            "move",
            CanvasInputOwnership {
                pointer: true,
                ..none
            },
            none,
        ),
        ("resize", none, CanvasInputOwnership { ime: true, ..none }),
        (
            "marquee",
            CanvasInputOwnership {
                scroll: true,
                ..none
            },
            none,
        ),
        (
            "move",
            none,
            CanvasInputOwnership {
                touch: true,
                ..none
            },
        ),
        (
            "resize",
            CanvasInputOwnership {
                keyboard: true,
                ..none
            },
            none,
        ),
    ] {
        let session = session()?;
        let original = session.document().raw_json().to_owned();
        let ctx = egui::Context::default();
        let mut view = CanvasView::default();
        let mut host = TakeoverInput::default();
        let initial = takeover_frame(&ctx, &mut view, &session, &mut host, vec![])?;
        let rect = initial.nodes[0].reserved;
        let start = match kind {
            "resize" => rect.right_bottom() - Vec2::splat(4.),
            "marquee" => rect.left_top() - Vec2::splat(24.),
            _ => rect.center(),
        };
        takeover_frame(&ctx, &mut view, &session, &mut host, press(start))?;
        assert!(view.has_gesture(), "{kind} starts");
        let moved = start + Vec2::new(24., 12.);
        takeover_frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            vec![Event::PointerMoved(moved)],
        )?;
        host.host = host_owner;
        host.config = config_owner;
        let blocked = takeover_frame(&ctx, &mut view, &session, &mut host, vec![])?;
        assert!(blocked.intents.is_empty());
        assert!(!view.has_gesture(), "{kind} ends on merged ownership");
        host.host = none;
        host.config = none;
        let released = takeover_frame(&ctx, &mut view, &session, &mut host, release(moved))?;
        assert!(released.intents.is_empty());
        assert!(!view.has_gesture());
        assert_eq!(session.document().raw_json(), original);
        assert_eq!(session.history_usage().undo_steps, 0);
        assert_eq!(session.history_usage().redo_steps, 0);
    }
    Ok(())
}

#[test]
fn exclusions_end_move_resize_and_marquee_without_reviving_a_blocked_release() -> TestResult {
    for kind in ["move", "resize", "marquee"] {
        let mut session = session()?;
        let original = session.document().raw_json().to_owned();
        let ctx = egui::Context::default();
        let mut view = CanvasView::default();
        let mut host = TakeoverInput::default();
        let initial = takeover_frame(&ctx, &mut view, &session, &mut host, vec![])?;
        let rect = initial.nodes[0].reserved;
        let start = match kind {
            "resize" => rect.right_bottom() - Vec2::splat(4.),
            "marquee" => rect.left_top() - Vec2::splat(24.),
            _ => rect.center(),
        };
        takeover_frame(&ctx, &mut view, &session, &mut host, press(start))?;
        let moved = start + Vec2::new(24., 12.);
        takeover_frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            vec![Event::PointerMoved(moved)],
        )?;
        assert!(view.has_gesture());
        host.exclusions = vec![Rect::from_center_size(moved, Vec2::splat(8.))];
        let events = match kind {
            // The current accepted position is now covered, without pointer movement.
            "resize" => vec![],
            // Crossing before release cancels even when the release and a later cycle are clear.
            "marquee" => {
                host.exclusions = vec![Rect::from_center_size(
                    moved + Vec2::splat(16.),
                    Vec2::splat(8.),
                )];
                let mut events = vec![Event::PointerMoved(moved + Vec2::splat(16.))];
                events.extend(release(moved + Vec2::splat(40.)));
                events.extend(press(rect.center()));
                events.extend(release(rect.center() + Vec2::splat(12.)));
                events
            }
            _ => release(moved),
        };
        let blocked = takeover_frame(&ctx, &mut view, &session, &mut host, events)?;
        assert!(blocked.intents.is_empty());
        assert!(!view.has_gesture(), "{kind} must not remain captured");
        host.exclusions.clear();
        let after = takeover_frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            release(moved + Vec2::splat(48.)),
        )?;
        assert!(after.intents.is_empty());
        let hover = takeover_frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            vec![Event::PointerMoved(rect.center())],
        )?;
        assert!(hover.intents.is_empty());
        assert!(!view.has_gesture());
        assert_eq!(session.document().raw_json(), original);
        assert_eq!(session.history_usage().undo_steps, 0);
        assert_eq!(session.history_usage().redo_steps, 0);

        takeover_frame(&ctx, &mut view, &session, &mut host, press(rect.center()))?;
        let clean = takeover_frame(
            &ctx,
            &mut view,
            &session,
            &mut host,
            release(rect.center() + Vec2::new(12., 6.)),
        )?;
        assert_eq!(clean.intents.len(), 1);
        let transaction = clean
            .intents
            .into_iter()
            .find_map(|intent| match intent {
                CanvasIntent::Transaction(transaction) => Some(transaction),
                _ => None,
            })
            .ok_or("clean recovery transaction")?;
        let prepared = session.prepare(transaction, &StandardDialect)?;
        session.apply(prepared)?;
        assert_eq!(session.document().node("a").ok_or("a")?.bounds().x, 12.);
        assert_eq!(session.history_usage().undo_steps, 1);
        let undo = session.prepare_undo()?;
        session.apply(undo)?;
        assert_eq!(session.document().raw_json(), original);
    }
    Ok(())
}

#[test]
fn excluded_hover_after_matching_release_does_not_cancel_completed_geometry() -> TestResult {
    for resize in [false, true] {
        let mut session = session()?;
        let original = session.document().raw_json().to_owned();
        let ctx = egui::Context::default();
        let mut view = CanvasView::default();
        let mut host = TakeoverInput::default();
        let initial = takeover_frame(&ctx, &mut view, &session, &mut host, vec![])?;
        let rect = initial.nodes[0].reserved;
        let start = if resize {
            rect.right_bottom() - Vec2::splat(4.)
        } else {
            rect.center()
        };
        let end = start + Vec2::new(24., 12.);
        let later = end + Vec2::splat(80.);
        host.exclusions = vec![Rect::from_center_size(later, Vec2::splat(8.))];
        let mut events = press(start);
        events.extend(release(end));
        events.push(Event::PointerMoved(later));
        events.push(if resize {
            Event::Zoom(1.5)
        } else {
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0., -24.),
                phase: egui::TouchPhase::Move,
                modifiers: Modifiers::NONE,
            }
        });
        events.push(Event::Key {
            key: egui::Key::Delete,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        });
        let camera = view.camera();
        let completed = takeover_frame(&ctx, &mut view, &session, &mut host, events)?;
        assert!(!view.has_gesture());
        assert_eq!(completed.intents.len(), 1);
        assert_eq!(view.camera(), camera);
        if !resize {
            assert_ne!(ctx.input(|input| input.smooth_scroll_delta), Vec2::ZERO);
        }
        let transaction = completed
            .intents
            .into_iter()
            .find_map(|intent| match intent {
                CanvasIntent::Transaction(transaction) => Some(transaction),
                _ => None,
            })
            .ok_or("completed geometry transaction")?;
        let prepared = session.prepare(transaction, &StandardDialect)?;
        session.apply(prepared)?;
        let bounds = session.document().node("a").ok_or("a")?.bounds();
        assert_eq!(
            (bounds.x, bounds.y, bounds.width, bounds.height),
            if resize {
                (0., 0., 224., 132.)
            } else {
                (24., 12., 200., 120.)
            }
        );
        assert_eq!(session.history_usage().undo_steps, 1);
        let undo = session.prepare_undo()?;
        session.apply(undo)?;
        assert_eq!(session.document().raw_json(), original);
    }
    Ok(())
}
