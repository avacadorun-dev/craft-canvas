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

fn pending_node(session: &CanvasSession) -> Result<NodeRecord> {
    NodeRecord::parse(
        r#"{"id":"pending","type":"text","x":20,"y":20,"width":300,"height":180,"text":"","future":{"preserve":9007199254740993}}"#,
        session.limits(),
    )
}
fn insert_pending(session: &CanvasSession, node: NodeRecord) -> Result<PreparedChange> {
    session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::InsertNode {
                node,
                index: session.document().nodes().len(),
            }],
        ),
        &StandardDialect,
    )
}
#[test]
fn pending_card_should_own_next_frame_text_and_native_undo_across_insertion_ack() -> TestResult {
    let mut session = session()?;
    let before = session.document().raw_json().to_owned();
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    frame(&ctx, &mut editor, vec![], 0., true, 1.);
    let typed = frame(
        &ctx,
        &mut editor,
        vec![Event::Text("Immediate 日本語 e\u{301} 😀".into())],
        1.,
        true,
        1.,
    );
    assert!(typed.changed && typed.input.keyboard_captured);
    assert_eq!(editor.text(), "Immediate 日本語 e\u{301} 😀");
    assert!(editor.awaits_insertion());
    assert!(editor.submit().is_err());
    assert_eq!(session.document().raw_json(), before);
    frame(&ctx, &mut editor, vec![], 2., true, 1.);
    session.apply(insert_pending(&session, pending_node(&session)?)?)?;
    editor.acknowledge_insertion(&session)?;
    assert!(!editor.awaits_insertion());
    assert_eq!(editor.text(), "Immediate 日本語 e\u{301} 😀");
    assert!(editor.dirty());
    frame(
        &ctx,
        &mut editor,
        vec![key(Key::Z, Modifiers::COMMAND)],
        3.,
        true,
        1.,
    );
    assert_eq!(editor.text(), "");
    frame(
        &ctx,
        &mut editor,
        vec![key(
            Key::Z,
            Modifiers {
                shift: true,
                ..Modifiers::COMMAND
            },
        )],
        4.,
        true,
        1.,
    );
    assert_eq!(editor.text(), "Immediate 日本語 e\u{301} 😀");
    session.apply(session.prepare(editor.submit()?, &StandardDialect)?)?;
    editor.acknowledge(&session)?;
    assert!(!editor.dirty());
    assert_eq!(
        session.document().node("pending").ok_or("pending")?.body(),
        "Immediate 日本語 e\u{301} 😀"
    );
    assert!(
        session
            .document()
            .node("pending")
            .ok_or("pending")?
            .raw_json()
            .contains("9007199254740993")
    );
    Ok(())
}
#[test]
fn pending_insertion_should_refuse_foreign_stale_wrong_record_and_existing_id_without_losing_draft()
-> TestResult {
    let mut current = session()?;
    let mut editor = TextCardEditor::pending_insertion(&current, pending_node(&current)?)?;
    *editor.text_mut() = "Retained exact draft 🪶".into();
    let mut foreign = session()?;
    foreign.apply(insert_pending(&foreign, pending_node(&foreign)?)?)?;
    assert!(matches!(
        editor.acknowledge_insertion(&foreign),
        Err(CanvasError::StaleBase)
    ));
    let wrong = NodeRecord::parse(
        &pending_node(&current)?
            .raw_json()
            .replace("\"width\":300", "\"width\":301"),
        current.limits(),
    )?;
    current.apply(insert_pending(&current, wrong)?)?;
    assert!(matches!(
        editor.acknowledge_insertion(&current),
        Err(CanvasError::StaleBase)
    ));
    assert_eq!(editor.text(), "Retained exact draft 🪶");
    assert!(editor.awaits_insertion());
    assert!(editor.pending_node(&current).is_none());
    assert!(TextCardEditor::pending_insertion(&current, pending_node(&current)?).is_err());
    let mut current = session()?;
    let mut editor = TextCardEditor::pending_insertion(&current, pending_node(&current)?)?;
    *editor.text_mut() = "Later retained".into();
    current.apply(insert_pending(&current, pending_node(&current)?)?)?;
    let extra = current.prepare(
        CanvasTransaction::new(
            current.identity(),
            vec![CanvasCommand::EditText {
                node_id: "a".into(),
                expected: "Original 😀".into(),
                text: "Other edit".into(),
            }],
        ),
        &StandardDialect,
    )?;
    current.apply(extra)?;
    assert!(matches!(
        editor.acknowledge_insertion(&current),
        Err(CanvasError::StaleBase)
    ));
    assert_eq!(editor.text(), "Later retained");
    Ok(())
}
#[test]
fn pending_surface_should_use_the_real_card_host_without_mutating_canonical_or_emitting_intents()
-> TestResult {
    struct Host {
        editor: TextCardEditor,
        typed: bool,
    }
    impl craft_canvas_egui::CanvasViewHost for Host {
        fn pending_text_editor(&self) -> Option<&TextCardEditor> {
            Some(&self.editor)
        }
        fn paint_body(
            &mut self,
            ui: &mut egui::Ui,
            node: &NodeRecord,
            rect: Rect,
            zoom: f64,
        ) -> craft_canvas_egui::NodeBodyResponse {
            if node.id() != self.editor.node_id() {
                return Default::default();
            }
            let output = self.editor.show_inline(ui, rect, zoom);
            self.typed |= output.changed;
            output.input
        }
    }
    let session = session()?;
    let before = session.document().raw_json().to_owned();
    let mut host = Host {
        editor: TextCardEditor::pending_insertion(&session, pending_node(&session)?)?,
        typed: false,
    };
    let mut view = craft_canvas_egui::CanvasView::default();
    let ctx = Context::default();
    for (index, events) in [vec![], vec![Event::Text("Next frame".into())]]
        .into_iter()
        .enumerate()
    {
        let mut full = ctx.run_ui(
            RawInput {
                events,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
                time: Some(index as f64),
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    let mut config = craft_canvas_egui::CanvasViewConfig::new(
                        egui::Id::new("pending-board"),
                        ui,
                    );
                    config.interactive = false;
                    config.shortcuts_blocked = true;
                    let output = view.show(ui, &session, &StandardDialect, &mut host, &config);
                    assert!(output.intents.is_empty());
                    assert!(output.errors.is_empty());
                    assert_eq!(
                        output
                            .nodes
                            .iter()
                            .filter(|node| node.node_id == "pending")
                            .count(),
                        1
                    );
                });
            },
        );
        full.textures_delta.clear();
    }
    assert!(host.typed);
    assert_eq!(host.editor.text(), "Next frame");
    assert_eq!(session.document().raw_json(), before);
    Ok(())
}
