//! Actual widget regressions for previously unpainted markers and unreachable edge deletion.
use craft_canvas_egui::*;
use craft_canvas_model::*;
use egui::{Event, Id, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

const RAW: &str = r#"{"nodes":[{"id":"a","type":"text","x":-300,"y":0,"width":100,"height":100,"text":"A"},{"id":"b","type":"text","x":100,"y":100,"width":100,"height":100,"text":"B"}],"edges":[{"id":"edge","fromNode":"a","fromSide":"right","fromEnd":"arrow","toNode":"b","toSide":"left","label":"preserved","future":{"number":1.23000}}]}"#;
fn session(raw: &str) -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(CanvasDocument::parse(raw, &limits)?, limits)
}
fn frame(
    ctx: &egui::Context,
    view: &mut CanvasView,
    session: &CanvasSession,
    events: Vec<Event>,
) -> Result<(CanvasViewOutput, usize)> {
    frame_with_host_block(ctx, view, session, events, false)
}
fn frame_with_host_block(
    ctx: &egui::Context,
    view: &mut CanvasView,
    session: &CanvasSession,
    events: Vec<Event>,
    blocked: bool,
) -> Result<(CanvasViewOutput, usize)> {
    let mut output = None;
    let mut rendered = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 680.0))),
            focused: true,
            events,
            ..Default::default()
        },
        |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut config = CanvasViewConfig::new(Id::new("edge-regression"), ui);
                config.shortcuts_blocked = blocked;
                output =
                    Some(view.show(ui, session, &StandardDialect, &mut PlainTextBody, &config));
            });
        },
    );
    let triangles = rendered
        .shapes
        .iter()
        .map(|shape| triangle_count(&shape.shape))
        .sum();
    rendered.textures_delta.clear();
    rendered.platform_output.commands.clear();
    Ok((
        output.ok_or_else(|| CanvasError::Invalid("missing native frame".into()))?,
        triangles,
    ))
}
fn triangle_count(shape: &egui::Shape) -> usize {
    match shape {
        egui::Shape::Path(path) if path.closed && path.points.len() == 3 => 1,
        egui::Shape::Vec(shapes) => shapes.iter().map(triangle_count).sum(),
        _ => 0,
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

#[test]
fn painted_routed_edge_selects_and_deletes_without_removing_cards_then_undo_restores_raw_fields()
-> TestResult {
    let mut session = session(RAW)?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let (first, triangles) = frame(&ctx, &mut view, &session, vec![])?;
    assert!(first.errors.is_empty());
    let edge = first.edges.first().ok_or("visible edge")?;
    assert!(edge.points.len() > 4 && edge.source_arrow.is_some() && edge.target_arrow.is_some());
    assert_eq!(
        triangles, 2,
        "both requested endpoint markers must actually be painted"
    );
    let hit = edge.label_anchor;
    frame(&ctx, &mut view, &session, pointer(hit, true))?;
    frame(&ctx, &mut view, &session, pointer(hit, false))?;
    assert_eq!(
        view.selection().edges.iter().next().map(String::as_str),
        Some("edge")
    );
    assert!(view.selection().nodes.is_empty());
    let (deleted, _) = frame(
        &ctx,
        &mut view,
        &session,
        vec![Event::Key {
            key: Key::Delete,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    )?;
    let transaction = deleted
        .intents
        .into_iter()
        .find_map(|intent| match intent {
            CanvasIntent::Transaction(transaction) => Some(transaction),
            _ => None,
        })
        .ok_or("edge-only deletion transaction")?;
    let prepared = session.prepare(transaction, &StandardDialect)?;
    session.apply(prepared)?;
    assert!(session.document().edges().is_empty());
    assert_eq!(session.document().nodes().len(), 2);
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), RAW);
    Ok(())
}

#[test]
fn stored_same_card_omitted_sides_have_a_visible_selectable_loop_without_changing_data()
-> TestResult {
    let raw = r#"{"nodes":[{"id":"a","type":"text","x":-50,"y":-50,"width":100,"height":100,"text":"A"}],"edges":[{"id":"loop","fromNode":"a","toNode":"a","toEnd":"none"}]}"#;
    let session = session(raw)?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let (first, triangles) = frame(&ctx, &mut view, &session, vec![])?;
    let edge = first.edges.first().ok_or("visible legacy loop")?;
    assert!(first.errors.is_empty() && edge.points.len() > 8);
    assert_eq!(triangles, 0);
    let card = first.nodes.first().ok_or("card")?.reserved;
    assert!(!card.contains(edge.label_anchor));
    let hit = edge.label_anchor;
    frame(&ctx, &mut view, &session, pointer(hit, true))?;
    frame(&ctx, &mut view, &session, pointer(hit, false))?;
    assert!(view.selection().edges.contains("loop"));
    // A node remains the topmost hit even where the connection touches its boundary.
    let shared_endpoint = *edge.points.first().ok_or("painted loop endpoint")?;
    assert!(card.contains(shared_endpoint));
    frame(&ctx, &mut view, &session, pointer(shared_endpoint, true))?;
    assert!(view.selection().nodes.contains("a") && view.selection().edges.is_empty());
    assert_eq!(session.document().raw_json(), raw);
    Ok(())
}

#[test]
fn host_blocked_input_cannot_select_or_delete_an_edge() -> TestResult {
    let session = session(RAW)?;
    let ctx = egui::Context::default();
    let mut view = CanvasView::default();
    let (first, _) = frame(&ctx, &mut view, &session, vec![])?;
    let hit = first.edges.first().ok_or("visible edge")?.label_anchor;
    frame_with_host_block(&ctx, &mut view, &session, pointer(hit, true), true)?;
    let (blocked_release, _) =
        frame_with_host_block(&ctx, &mut view, &session, pointer(hit, false), true)?;
    assert!(view.selection().edges.is_empty() && view.selection().nodes.is_empty());
    assert!(blocked_release.intents.is_empty() && !view.has_gesture());

    // Establish real edge selection and board focus before testing the host's keyboard block.
    frame(&ctx, &mut view, &session, pointer(hit, true))?;
    frame(&ctx, &mut view, &session, pointer(hit, false))?;
    assert!(view.selection().edges.contains("edge"));
    let (blocked_delete, _) = frame_with_host_block(
        &ctx,
        &mut view,
        &session,
        vec![Event::Key {
            key: Key::Delete,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        true,
    )?;
    assert!(blocked_delete.intents.is_empty());
    assert!(view.selection().edges.contains("edge"));
    assert_eq!(session.document().raw_json(), RAW);
    Ok(())
}
