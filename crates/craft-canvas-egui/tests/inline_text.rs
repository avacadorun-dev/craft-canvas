//! Actual native input-event and acknowledgement contracts; OS/host acceptance is separate.
use craft_canvas_egui::{CanvasViewLabels, InlineTextOutput, TextCardEditor};
use craft_canvas_model::*;
use egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(
        CanvasDocument::parse(
            r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":300,"height":180,"text":"Original 😀","future":{"keep":true}}],"edges":[],"future":{"preserve":true}}"#,
            &limits,
        )?,
        limits,
    )
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
fn frame(
    ctx: &Context,
    editor: &mut TextCardEditor,
    events: Vec<Event>,
    time: f64,
    focused: bool,
    zoom: f64,
) -> InlineTextOutput {
    let mut output = InlineTextOutput::default();
    let mut full = ctx.run_ui(
        RawInput {
            events,
            time: Some(time),
            focused,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
            ..Default::default()
        },
        |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let rect = Rect::from_min_size(Pos2::new(40., 40.), Vec2::new(300., 180.));
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.set_clip_rect(rect);
                    output = editor.show_inline(ui, rect, zoom);
                });
            });
        },
    );
    full.textures_delta.clear();
    full.platform_output.commands.clear();
    output
}
#[test]
fn inline_widget_should_receive_native_selection_paste_and_local_undo_without_a_canonical_write()
-> TestResult {
    let session = session()?;
    let original = session.document().raw_json().to_owned();
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    frame(&ctx, &mut editor, vec![], 0., true, 1.);
    let output = frame(
        &ctx,
        &mut editor,
        vec![
            key(Key::A, Modifiers::COMMAND),
            Event::Paste("日本語 e\u{301} 🪶\n**Markdown**".into()),
        ],
        1.,
        true,
        1.,
    );
    assert!(output.changed);
    assert!(output.input.keyboard_captured);
    assert_eq!(editor.text(), "日本語 e\u{301} 🪶\n**Markdown**");
    assert!(editor.dirty());
    frame(&ctx, &mut editor, vec![], 2., true, 1.);
    frame(
        &ctx,
        &mut editor,
        vec![key(Key::Z, Modifiers::COMMAND)],
        3.,
        true,
        1.,
    );
    assert_eq!(editor.text(), "Original 😀");
    assert_eq!(session.document().raw_json(), original);
    Ok(())
}
#[test]
fn submitting_then_typing_more_should_keep_later_draft_during_exact_acknowledgement() -> TestResult
{
    let mut session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    *editor.text_mut() = "First saved source".into();
    let transaction = editor.submit()?;
    let prepared = session.prepare(transaction, &StandardDialect)?;
    *editor.text_mut() = "First saved source; later 日本語".into();
    assert!(editor.submit().is_err());
    session.apply(prepared)?;
    editor.acknowledge(&session)?;
    assert_eq!(editor.text(), "First saved source; later 日本語");
    assert!(editor.dirty());
    assert!(!editor.has_submission());
    let next = session.prepare(editor.submit()?, &StandardDialect)?;
    session.apply(next)?;
    editor.acknowledge(&session)?;
    assert!(!editor.dirty());
    assert!(
        session
            .document()
            .raw_json()
            .contains("\"future\":{\"keep\":true}")
    );
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(
        session.document().node("a").ok_or("a")?.body(),
        "First saved source"
    );
    Ok(())
}
#[test]
fn foreign_or_wrong_source_acknowledgement_should_preserve_the_entire_local_submission()
-> TestResult {
    let mut session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    *editor.text_mut() = "Exact pending source".into();
    let submitted = editor.submit()?;
    let foreign = self::session()?;
    assert!(matches!(
        editor.acknowledge(&foreign),
        Err(CanvasError::StaleBase)
    ));
    let other = session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::EditText {
                node_id: "a".into(),
                expected: "Original 😀".into(),
                text: "Other source".into(),
            }],
        ),
        &StandardDialect,
    )?;
    session.apply(other)?;
    assert!(matches!(
        editor.acknowledge(&session),
        Err(CanvasError::StaleBase)
    ));
    assert!(editor.has_submission());
    assert_eq!(editor.text(), "Exact pending source");
    assert!(matches!(
        session.prepare(submitted, &StandardDialect),
        Err(CanvasError::StaleBase)
    ));
    Ok(())
}
#[test]
fn native_composition_or_os_focus_loss_should_not_be_done_or_cancel() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    frame(&ctx, &mut editor, vec![], 0., true, 0.5);
    let output = frame(
        &ctx,
        &mut editor,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "文字".into(),
                active_range_chars: Some(0..2),
            }),
            key(Key::Enter, Modifiers::COMMAND),
            key(Key::Escape, Modifiers::NONE),
        ],
        1.,
        true,
        0.5,
    );
    assert!(output.composing);
    assert!(!output.done_requested);
    assert!(!output.cancelled);
    let output = frame(&ctx, &mut editor, vec![], 2., false, 2.);
    assert!(!output.done_requested);
    assert!(!output.cancelled);
    frame(
        &ctx,
        &mut editor,
        vec![Event::Ime(egui::ImeEvent::Commit("文字".into()))],
        3.,
        true,
        1.,
    );
    let pos = Pos2::new(80., 80.);
    frame(
        &ctx,
        &mut editor,
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
        4.,
        true,
        1.,
    );
    frame(
        &ctx,
        &mut editor,
        vec![Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
        4.5,
        true,
        1.,
    );
    let output = frame(
        &ctx,
        &mut editor,
        vec![key(Key::Enter, Modifiers::COMMAND)],
        5.,
        true,
        1.,
    );
    assert!(output.done_requested);
    assert!(!output.cancelled);
    Ok(())
}
#[test]
fn inline_and_optional_source_pane_share_one_retained_exact_draft() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    *editor.text_mut() = "Shared draft".into();
    let ctx = Context::default();
    frame(&ctx, &mut editor, vec![], 0., true, f64::NAN);
    let mut frame = ctx.run_ui(RawInput::default(), |root| {
        egui::CentralPanel::default().show(root, |ui| {
            let _ = editor.show(ui, &CanvasViewLabels::default());
        });
    });
    frame.textures_delta.clear();
    frame.platform_output.commands.clear();
    assert_eq!(editor.text(), "Shared draft");
    assert!(editor.dirty());
    Ok(())
}
