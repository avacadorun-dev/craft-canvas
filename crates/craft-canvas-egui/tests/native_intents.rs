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
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let config = CanvasViewConfig::new(Id::new("synthetic-board"), ui);
            output = Some(view.show(ui, session, &StandardDialect, host, &config));
        });
    });
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
