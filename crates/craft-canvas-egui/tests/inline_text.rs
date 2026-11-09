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
/// Release renderer/platform work in headless tests while retaining native IME evidence.
fn finish_headless_frame(mut output: egui::FullOutput) -> egui::FullOutput {
    output.textures_delta.clear();
    output.platform_output.commands.clear();
    output
}
fn frame(
    ctx: &Context,
    editor: &mut TextCardEditor,
    events: Vec<Event>,
    time: f64,
    focused: bool,
    zoom: f64,
) -> InlineTextOutput {
    frame_with_platform(ctx, editor, events, time, focused, zoom, true).0
}

fn frame_with_platform(
    ctx: &Context,
    editor: &mut TextCardEditor,
    events: Vec<Event>,
    time: f64,
    focused: bool,
    zoom: f64,
    enabled: bool,
) -> (InlineTextOutput, egui::FullOutput) {
    let mut output = InlineTextOutput::default();
    let mut raw = RawInput {
        events,
        time: Some(time),
        focused,
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
        ..Default::default()
    };
    editor.prepare_inline_input(ctx, &mut raw, !enabled);
    let full = finish_headless_frame(ctx.run_ui(raw, |root| {
        egui::CentralPanel::default().show(root, |ui| {
            let rect = Rect::from_min_size(Pos2::new(40., 40.), Vec2::new(300., 180.));
            ui.add_enabled_ui(enabled, |ui| {
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.set_clip_rect(rect);
                    output = editor.show_inline(ui, rect, zoom);
                });
            });
        });
    }));
    (output, full)
}

/// A real prior input and creation command; the new card has not been painted yet.
fn creation_handoff(ctx: &Context, editor: &mut TextCardEditor) -> TestResult {
    creation_handoff_frames(ctx, editor, false)?;
    Ok(())
}

fn creation_handoff_frames(
    ctx: &Context,
    editor: &mut TextCardEditor,
    mount_during_creation: bool,
) -> std::result::Result<Vec<egui::FullOutput>, Box<dyn std::error::Error>> {
    let mut frames = Vec::new();
    let mut previous_text = String::from("Previous input");
    let previous_id = egui::Id::new("previous-input");
    let mut activation = None;
    for (time, create) in [(-2., false), (-1., true)] {
        let full = finish_headless_frame(ctx.run_ui(
            RawInput {
                events: if create {
                    vec![key(Key::N, Modifiers::COMMAND)]
                } else {
                    vec![]
                },
                time: Some(time),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    if !create {
                        ui.memory_mut(|memory| memory.request_focus(previous_id));
                    }
                    ui.add(egui::TextEdit::singleline(&mut previous_text).id(previous_id));
                    if ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::N)) {
                        activation = Some(editor.begin_inline(ui.ctx()));
                        if mount_during_creation {
                            let rect =
                                Rect::from_min_size(Pos2::new(40., 40.), Vec2::new(300., 180.));
                            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                editor.show_inline(ui, rect, 1.);
                            });
                        }
                    }
                });
            },
        ));
        // A real prior text input enabled native IME before the creation handoff.
        assert!(full.platform_output.ime.is_some());
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(previous_id));
        frames.push(full);
    }
    activation.ok_or("creation command was not handled")??;
    Ok(frames)
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
    let _frame = finish_headless_frame(ctx.run_ui(RawInput::default(), |root| {
        egui::CentralPanel::default().show(root, |ui| {
            let _ = editor.show(ui, &CanvasViewLabels::default());
        });
    }));
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
        output: InlineTextOutput,
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
            let input = output.input;
            self.output = output;
            input
        }
    }
    let session = session()?;
    let before = session.document().raw_json().to_owned();
    let mut host = Host {
        editor: TextCardEditor::pending_insertion(&session, pending_node(&session)?)?,
        output: InlineTextOutput::default(),
    };
    let mut view = craft_canvas_egui::CanvasView::default();
    let ctx = Context::default();
    creation_handoff(&ctx, &mut host.editor)?;
    // No bootstrap card paint: CreateText mounts native input after its creation intent.
    // Withholding acknowledgement also exercises a retained draft during a save failure.
    for (index, events) in [
        vec![Event::Text("Next frame".into())],
        vec![Event::Text(" before acknowledgement".into())],
        vec![key(Key::Escape, Modifiers::NONE)],
    ]
    .into_iter()
    .enumerate()
    {
        let mut raw = RawInput {
            events,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
            time: Some(index as f64),
            ..Default::default()
        };
        host.editor.prepare_inline_input(&ctx, &mut raw, false);
        let full = finish_headless_frame(ctx.run_ui(raw, |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut config =
                    craft_canvas_egui::CanvasViewConfig::new(egui::Id::new("pending-board"), ui);
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
        }));
        assert_eq!(full.platform_output.ime.is_some(), index != 2);
        assert!(host.output.input.keyboard_captured);
        assert!(!host.output.done_requested);
        assert_eq!(host.output.cancelled, index == 2);
        if index == 0 {
            assert_eq!(host.editor.text(), "Next frame");
        }
    }
    assert_eq!(host.editor.text(), "Next frame before acknowledgement");
    assert!(host.editor.awaits_insertion());
    assert!(host.editor.submit().is_err());
    assert_eq!(session.document().raw_json(), before);
    Ok(())
}

#[test]
fn creating_pointer_should_keep_initial_card_focus_without_requesting_done() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    let pos = Pos2::new(600., 20.);
    let output = frame(
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
        0.,
        true,
        1.,
    );
    assert!(output.input.keyboard_captured);
    assert!(!output.done_requested);
    assert!(!output.cancelled);
    frame(
        &ctx,
        &mut editor,
        vec![
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
            Event::Text("Immediately after creation".into()),
        ],
        1.,
        true,
        1.,
    );
    assert_eq!(editor.text(), "Immediately after creation");
    Ok(())
}

#[test]
fn first_mounted_ime_should_survive_native_focus_handoff_and_insertion_acknowledgement()
-> TestResult {
    let mut session = session()?;
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    let (output, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![
            Event::Text("Selected source 🪶".into()),
            key(Key::A, Modifiers::COMMAND),
            Event::Ime(egui::ImeEvent::Preedit {
                text: "にほん".into(),
                active_range_chars: Some(0..3),
            }),
            key(Key::Enter, Modifiers::COMMAND),
            key(Key::Escape, Modifiers::NONE),
        ],
        0.,
        true,
        1.,
        true,
    );
    assert!(output.composing && output.input.keyboard_captured);
    assert!(!output.done_requested && !output.cancelled);
    assert!(
        !full
            .platform_output
            .ime
            .ok_or("native IME output")?
            .should_interrupt_composition
    );
    assert_eq!(editor.text(), "にほん");
    session.apply(insert_pending(&session, pending_node(&session)?)?)?;
    editor.acknowledge_insertion(&session)?;
    let (output, full) = frame_with_platform(&ctx, &mut editor, vec![], 1., true, 1., true);
    assert!(output.composing && output.input.keyboard_captured);
    assert!(
        !full
            .platform_output
            .ime
            .ok_or("retained native IME output")?
            .should_interrupt_composition
    );
    assert_eq!(editor.text(), "にほん");
    let (_, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![Event::Ime(egui::ImeEvent::Commit("日本語".into()))],
        2.,
        true,
        1.,
        true,
    );
    assert!(
        !full
            .platform_output
            .ime
            .ok_or("committed native IME output")?
            .should_interrupt_composition
    );
    assert_eq!(editor.text(), "日本語");
    let output = frame(
        &ctx,
        &mut editor,
        vec![key(Key::Enter, Modifiers::COMMAND)],
        3.,
        true,
        1.,
    );
    assert!(output.done_requested && !output.composing);
    let output = frame(
        &ctx,
        &mut editor,
        vec![key(Key::Escape, Modifiers::NONE)],
        4.,
        true,
        1.,
    );
    assert!(output.cancelled && !output.done_requested);
    assert!(!frame(&ctx, &mut editor, vec![], 5., true, 1.).cancelled);
    assert_eq!(
        session.document().node("pending").ok_or("pending")?.body(),
        ""
    );
    Ok(())
}

#[test]
fn pending_draft_preservation_should_prepare_original_insert_and_latest_edit_atomically()
-> TestResult {
    let session = session()?;
    let before = session.document().raw_json().to_owned();
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    *editor.text_mut() = "Latest unacknowledged 日本語 e\u{301} 🪶".into();
    let pending = editor.pending_node(&session).ok_or("pending draft")?;
    let preserved = session.prepare(
        CanvasTransaction::new(
            editor.base(),
            vec![
                CanvasCommand::InsertNode {
                    node: NodeRecord::parse(pending.raw_json(), session.limits())?,
                    index: session.document().nodes().len(),
                },
                CanvasCommand::EditText {
                    node_id: editor.node_id().into(),
                    expected: pending.body().into(),
                    text: editor.text().into(),
                },
            ],
        ),
        &StandardDialect,
    )?;
    let roundtrip = CanvasDocument::parse(preserved.document().raw_json(), session.limits())?;
    assert_eq!(
        roundtrip.node("pending").ok_or("pending")?.body(),
        editor.text()
    );
    assert_eq!(roundtrip.nodes()[0].id(), "a");
    assert_eq!(roundtrip.nodes()[1].id(), "pending");
    assert!(
        roundtrip
            .node("pending")
            .ok_or("pending")?
            .raw_json()
            .contains("\"future\":{\"preserve\":9007199254740993}")
    );
    assert_eq!(
        roundtrip.node("a").ok_or("a")?.raw_json(),
        session.document().node("a").ok_or("a")?.raw_json()
    );
    assert_eq!(session.document().raw_json(), before);
    assert!(editor.awaits_insertion() && editor.dirty());
    assert!(editor.submit().is_err());
    Ok(())
}

#[test]
fn owned_escape_should_cancel_once_without_losing_the_pending_draft() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    let output = frame(
        &ctx,
        &mut editor,
        vec![key(Key::Escape, Modifiers::NONE)],
        0.,
        true,
        1.,
    );
    assert!(output.cancelled && !output.done_requested);
    assert!(editor.awaits_insertion());
    let output = frame(&ctx, &mut editor, vec![], 1., true, 1.);
    assert!(!output.cancelled);
    let repeated = Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: true,
        modifiers: Modifiers::NONE,
    };
    let output = frame(&ctx, &mut editor, vec![repeated], 2., true, 1.);
    assert!(!output.cancelled);
    assert!(editor.awaits_insertion());
    Ok(())
}

#[test]
fn raw_hook_should_leave_unowned_and_blocked_input_unchanged() -> TestResult {
    for guard in ["modal", "unfocused", "other-viewport", "other-field"] {
        let session = session()?;
        let mut editor = TextCardEditor::new(&session, "a")?;
        let ctx = Context::default();
        creation_handoff(&ctx, &mut editor)?;
        if guard == "other-field" {
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("other-field")));
        }
        let mut raw = RawInput {
            focused: guard != "unfocused",
            viewport_id: if guard == "other-viewport" {
                egui::ViewportId::from_hash_of("other-viewport")
            } else {
                egui::ViewportId::ROOT
            },
            events: vec![
                Event::Text("Text".into()),
                Event::Paste("Paste".into()),
                Event::Ime(egui::ImeEvent::Preedit {
                    text: "にほん".into(),
                    active_range_chars: Some(0..3),
                }),
                key(Key::Escape, Modifiers::NONE),
            ],
            ..Default::default()
        };
        let expected = raw.events.clone();
        editor.prepare_inline_input(&ctx, &mut raw, guard == "modal");
        assert_eq!(raw.events, expected, "{guard}");
        assert_eq!(editor.text(), "Original 😀");
    }
    Ok(())
}

#[test]
fn raw_hook_should_remove_only_owned_escape_and_preserve_native_input_order() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    let untouched = vec![
        Event::Text("Text".into()),
        Event::Paste("Paste".into()),
        Event::Ime(egui::ImeEvent::Preedit {
            text: "にほん".into(),
            active_range_chars: Some(0..3),
        }),
        key(Key::Escape, Modifiers::SHIFT),
    ];
    let mut raw = RawInput {
        events: untouched.clone(),
        ..Default::default()
    };
    raw.events.insert(1, key(Key::Escape, Modifiers::NONE));
    editor.prepare_inline_input(&ctx, &mut raw, false);
    assert_eq!(raw.events, untouched);
    editor.prepare_inline_input(&ctx, &mut raw, false);
    assert_eq!(raw.events, untouched);
    let output = frame(&ctx, &mut editor, vec![], 0., true, 1.);
    assert!(!output.cancelled);
    Ok(())
}

#[test]
fn disabled_retained_editor_should_not_process_text_paste_ime_or_shortcuts() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    frame(
        &ctx,
        &mut editor,
        vec![Event::Text("Retained draft".into())],
        0.,
        true,
        1.,
    );
    let (output, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![
            Event::Text(" unwanted".into()),
            Event::Paste("paste".into()),
            Event::Ime(egui::ImeEvent::Preedit {
                text: "にほん".into(),
                active_range_chars: Some(0..3),
            }),
            key(Key::Enter, Modifiers::COMMAND),
            key(Key::Escape, Modifiers::NONE),
        ],
        1.,
        true,
        1.,
        false,
    );
    assert_eq!(editor.text(), "Retained draft");
    assert!(!output.changed && !output.done_requested && !output.cancelled && !output.composing);
    assert!(
        !full
            .platform_output
            .ime
            .is_some_and(|ime| ime.should_interrupt_composition)
    );
    // egui may release a disabled widget; the retained editor must not reclaim it.
    assert!(ctx.memory(|memory| memory.focused()).is_none());
    Ok(())
}

fn navigation_frame(ctx: &Context, editor: &mut TextCardEditor, events: Vec<Event>, time: f64) {
    let mut raw = RawInput {
        events,
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
        ..Default::default()
    };
    editor.prepare_inline_input(ctx, &mut raw, false);
    let _full = finish_headless_frame(ctx.run_ui(raw, |root| {
        egui::CentralPanel::default().show(root, |ui| {
            ui.interact(
                Rect::from_min_size(Pos2::new(5., 40.), Vec2::new(25., 30.)),
                egui::Id::new("neighbor-before"),
                egui::Sense::click(),
            );
            let rect = Rect::from_min_size(Pos2::new(40., 40.), Vec2::new(300., 180.));
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.set_clip_rect(rect);
                editor.show_inline(ui, rect, 1.);
            });
            ui.interact(
                Rect::from_min_size(Pos2::new(400., 40.), Vec2::new(40., 30.)),
                egui::Id::new("neighbor-after"),
                egui::Sense::click(),
            );
        });
    }));
}

#[test]
fn initial_arrow_should_move_the_caret_without_moving_focus_to_a_neighbor() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    *editor.text_mut() = "ab".into();
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    // A pre-input hook can run more than once before the first visible mount.
    editor.prepare_inline_input(&ctx, &mut RawInput::default(), false);
    let card_id = ctx.memory(|memory| memory.focused()).ok_or("card focus")?;
    navigation_frame(
        &ctx,
        &mut editor,
        vec![key(Key::ArrowLeft, Modifiers::NONE)],
        0.,
    );
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(card_id));
    navigation_frame(&ctx, &mut editor, vec![Event::Text("X".into())], 1.);
    assert_eq!(editor.text(), "aXb");
    Ok(())
}

#[test]
fn initial_tab_and_shift_tab_should_keep_native_focus_navigation() -> TestResult {
    for (modifiers, target) in [
        (Modifiers::NONE, "neighbor-after"),
        (Modifiers::SHIFT, "neighbor-before"),
    ] {
        let session = session()?;
        let mut editor = TextCardEditor::new(&session, "a")?;
        let ctx = Context::default();
        creation_handoff(&ctx, &mut editor)?;
        navigation_frame(&ctx, &mut editor, vec![key(Key::Tab, modifiers)], 0.);
        // egui deliberately applies reverse focus navigation on the following pass.
        navigation_frame(&ctx, &mut editor, vec![], 1.);
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(egui::Id::new(target))
        );
    }
    Ok(())
}

#[test]
fn initial_accessibility_focus_should_not_be_overridden_by_arrow_ownership() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    let target = egui::Id::new("neighbor-after");
    navigation_frame(
        &ctx,
        &mut editor,
        vec![
            key(Key::ArrowLeft, Modifiers::NONE),
            Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target_node: target.accesskit_id(),
                target_tree: egui::accesskit::TreeId::ROOT,
                data: None,
            }),
        ],
        0.,
    );
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(target));
    Ok(())
}

#[test]
fn queued_escape_should_expire_when_ownership_is_blocked_before_mount() -> TestResult {
    for guard in ["disabled-mount", "other-field", "blocked-hook"] {
        let session = session()?;
        let mut editor = TextCardEditor::new(&session, "a")?;
        let ctx = Context::default();
        creation_handoff(&ctx, &mut editor)?;
        let mut raw = RawInput {
            events: vec![key(Key::Escape, Modifiers::NONE)],
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
            ..Default::default()
        };
        editor.prepare_inline_input(&ctx, &mut raw, false);
        assert!(raw.events.is_empty());
        if guard == "other-field" {
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("modal-input")));
        }
        if guard == "blocked-hook" {
            editor.prepare_inline_input(&ctx, &mut raw, true);
        }
        let mut cancelled = false;
        let _full = finish_headless_frame(ctx.run_ui(raw, |root| {
            egui::CentralPanel::default().show(root, |ui| {
                ui.add_enabled_ui(guard != "disabled-mount", |ui| {
                    let rect = Rect::from_min_size(Pos2::new(40., 40.), Vec2::new(300., 180.));
                    cancelled = editor.show_inline(ui, rect, 1.).cancelled;
                });
            });
        }));
        assert!(!cancelled, "{guard}");
        let output = frame(&ctx, &mut editor, vec![], 1., true, 1.);
        assert!(!output.cancelled, "{guard}");
        assert_eq!(editor.text(), "Original 😀");
    }
    Ok(())
}

#[test]
fn another_native_fields_composition_should_not_become_card_input_ownership() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    frame(&ctx, &mut editor, vec![], 0., true, 1.);
    let mut raw = RawInput {
        events: vec![Event::Ime(egui::ImeEvent::Preedit {
            text: "別の入力".into(),
            active_range_chars: Some(0..4),
        })],
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 500.))),
        time: Some(1.),
        ..Default::default()
    };
    let other_id = egui::Id::new("another-native-field");
    let mut other_text = String::new();
    editor.prepare_inline_input(&ctx, &mut raw, false);
    let mut output = InlineTextOutput::default();
    let _full = finish_headless_frame(ctx.run_ui(raw, |root| {
        egui::CentralPanel::default().show(root, |ui| {
            ui.memory_mut(|memory| memory.request_focus(other_id));
            ui.add(egui::TextEdit::singleline(&mut other_text).id(other_id));
            let rect = Rect::from_min_size(Pos2::new(40., 80.), Vec2::new(300., 180.));
            output = editor.show_inline(ui, rect, 1.);
        });
    }));
    assert_eq!(other_text, "別の入力");
    assert_eq!(editor.text(), "Original 😀");
    assert!(!output.composing && !output.input.keyboard_captured);
    assert!(!output.cancelled && !output.done_requested);
    Ok(())
}

#[test]
fn queued_escape_should_survive_eframe_deferred_hooks_until_exactly_one_mount() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    let mut raw = RawInput {
        events: vec![key(Key::Escape, Modifiers::NONE)],
        ..Default::default()
    };
    // eframe's logic-only update may retain post-hook raw input without mounting UI.
    editor.prepare_inline_input(&ctx, &mut raw, false);
    assert!(raw.events.is_empty());
    editor.prepare_inline_input(&ctx, &mut raw, false);
    let output = frame(&ctx, &mut editor, raw.events, 0., true, 1.);
    assert!(output.cancelled && !output.done_requested);
    assert!(!frame(&ctx, &mut editor, vec![], 1., true, 1.).cancelled);
    assert_eq!(editor.text(), "Original 😀");
    Ok(())
}

#[test]
fn owned_cancel_should_freeze_same_batch_text_paste_and_done_before_native_mount() -> TestResult {
    for cancel_first in [true, false] {
        let session = session()?;
        let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
        *editor.text_mut() = "Exact retained 日本語 e\u{301} 🪶".into();
        let retained = editor.text().to_owned();
        let ctx = Context::default();
        creation_handoff(&ctx, &mut editor)?;
        let mut events = vec![
            Event::Text(" unwanted text".into()),
            Event::Paste(" unwanted paste".into()),
            key(Key::Enter, Modifiers::COMMAND),
        ];
        let escape_index = if cancel_first { 0 } else { events.len() };
        events.insert(escape_index, key(Key::Escape, Modifiers::NONE));
        let (output, full) = frame_with_platform(&ctx, &mut editor, events, 0., true, 1., true);
        assert!(output.cancelled && output.input.keyboard_captured);
        assert!(!output.changed && !output.done_requested && !output.composing);
        assert_eq!(editor.text(), retained);
        assert!(ctx.memory(|memory| memory.focused()).is_none());
        assert!(full.platform_output.ime.is_none());
        assert!(!frame(&ctx, &mut editor, vec![], 1., true, 1.).cancelled);
        assert_eq!(editor.text(), retained);
        assert!(editor.awaits_insertion());
    }
    Ok(())
}

#[test]
fn current_frame_ime_should_override_a_deferred_cancel_without_freezing_native_input() -> TestResult
{
    let session = session()?;
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    creation_handoff(&ctx, &mut editor)?;
    let mut deferred = RawInput {
        events: vec![key(Key::Escape, Modifiers::NONE)],
        ..Default::default()
    };
    editor.prepare_inline_input(&ctx, &mut deferred, false);
    assert!(deferred.events.is_empty());
    let (output, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "にほん".into(),
                active_range_chars: Some(0..3),
            }),
            key(Key::Escape, Modifiers::NONE),
        ],
        0.,
        true,
        1.,
        true,
    );
    assert!(output.composing && output.input.keyboard_captured);
    assert!(!output.cancelled && !output.done_requested);
    assert_eq!(editor.text(), "にほん");
    assert!(
        !full
            .platform_output
            .ime
            .ok_or("native IME output")?
            .should_interrupt_composition
    );
    Ok(())
}

struct AccessibilityChanges;
impl accesskit_consumer::TreeChangeHandler for AccessibilityChanges {
    fn node_added(&mut self, _node: &accesskit_consumer::Node<'_>) {}
    fn node_updated(
        &mut self,
        _old: &accesskit_consumer::Node<'_>,
        _new: &accesskit_consumer::Node<'_>,
    ) {
    }
    fn focus_moved(
        &mut self,
        _old: Option<&accesskit_consumer::Node<'_>>,
        _new: Option<&accesskit_consumer::Node<'_>>,
    ) {
    }
    fn node_removed(&mut self, _node: &accesskit_consumer::Node<'_>) {}
}

fn accept_accessibility_update(
    tree: &mut Option<accesskit_consumer::Tree>,
    output: &egui::FullOutput,
) -> TestResult {
    let update = output
        .platform_output
        .accesskit_update
        .clone()
        .ok_or("AccessKit update")?;
    let focus = update.focus;
    let tree_id = update.tree_id;
    if let Some(tree) = tree {
        tree.update_and_process_changes(update, &mut AccessibilityChanges);
    } else {
        *tree = Some(accesskit_consumer::Tree::new(update, true));
    }
    let state = tree.as_ref().ok_or("AccessKit consumer")?.state();
    assert!(state.node_by_tree_local_id(focus, tree_id).is_some());
    assert!(state.focus().is_some());
    Ok(())
}

#[test]
fn late_activation_should_keep_accesskit_focus_valid_before_first_unicode_input() -> TestResult {
    for mount_during_creation in [false, true] {
        let session = session()?;
        let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut tree = None;
        for output in creation_handoff_frames(&ctx, &mut editor, mount_during_creation)? {
            accept_accessibility_update(&mut tree, &output)?;
        }
        let (output, full) = frame_with_platform(
            &ctx,
            &mut editor,
            vec![
                Event::Text("日本語 e\u{301} ".into()),
                Event::Paste("🪶 pasted".into()),
            ],
            0.,
            true,
            1.,
            true,
        );
        accept_accessibility_update(&mut tree, &full)?;
        assert!(output.changed && output.input.keyboard_captured);
        assert!(!output.done_requested && !output.cancelled);
        assert_eq!(editor.text(), "日本語 e\u{301} 🪶 pasted");
        assert_eq!(
            tree.as_ref()
                .ok_or("AccessKit tree")?
                .state()
                .focus_in_tree()
                .role(),
            egui::accesskit::Role::MultilineTextInput,
        );
    }
    Ok(())
}

#[test]
fn late_activation_should_keep_accesskit_and_ime_valid_through_ack_and_dismissal() -> TestResult {
    for dismiss in [Key::Escape, Key::Enter] {
        let mut session = session()?;
        let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut tree = None;
        for output in creation_handoff_frames(&ctx, &mut editor, false)? {
            accept_accessibility_update(&mut tree, &output)?;
        }
        let (output, full) = frame_with_platform(
            &ctx,
            &mut editor,
            vec![
                Event::Ime(egui::ImeEvent::Preedit {
                    text: "にほん".into(),
                    active_range_chars: Some(0..3),
                }),
                key(Key::Escape, Modifiers::NONE),
            ],
            0.,
            true,
            1.,
            true,
        );
        accept_accessibility_update(&mut tree, &full)?;
        assert!(output.composing && output.input.keyboard_captured);
        assert!(!output.cancelled && !output.done_requested);
        assert!(
            !full
                .platform_output
                .ime
                .ok_or("first native IME output")?
                .should_interrupt_composition
        );
        session.apply(insert_pending(&session, pending_node(&session)?)?)?;
        editor.acknowledge_insertion(&session)?;
        let (_, full) = frame_with_platform(
            &ctx,
            &mut editor,
            vec![Event::Ime(egui::ImeEvent::Commit("日本語".into()))],
            1.,
            true,
            1.,
            true,
        );
        accept_accessibility_update(&mut tree, &full)?;
        assert!(
            !full
                .platform_output
                .ime
                .ok_or("committed native IME output")?
                .should_interrupt_composition
        );
        assert_eq!(editor.text(), "日本語");
        let modifiers = if dismiss == Key::Enter {
            Modifiers::COMMAND
        } else {
            Modifiers::NONE
        };
        let (output, full) = frame_with_platform(
            &ctx,
            &mut editor,
            vec![key(dismiss, modifiers)],
            2.,
            true,
            1.,
            true,
        );
        accept_accessibility_update(&mut tree, &full)?;
        assert_eq!(output.cancelled, dismiss == Key::Escape);
        assert_eq!(output.done_requested, dismiss == Key::Enter);
        let full = finish_headless_frame(ctx.run_ui(RawInput::default(), |ui| {
            ui.label("Card input closed");
        }));
        accept_accessibility_update(&mut tree, &full)?;
        assert!(ctx.memory(|memory| memory.focused()).is_none());
    }
    Ok(())
}

#[test]
fn unmounted_pre_input_activation_should_not_leave_dangling_accesskit_focus() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut tree = None;
    for output in creation_handoff_frames(&ctx, &mut editor, false)? {
        accept_accessibility_update(&mut tree, &output)?;
    }
    let mut raw = RawInput::default();
    editor.prepare_inline_input(&ctx, &mut raw, false);
    // Even a host that omits the promised mount cannot emit a nonexistent focus node.
    let full = finish_headless_frame(ctx.run_ui(raw, |ui| {
        ui.label("The card is outside this view");
    }));
    accept_accessibility_update(&mut tree, &full)?;
    assert!(ctx.memory(|memory| memory.focused()).is_none());
    assert_eq!(editor.text(), "Original 😀");
    Ok(())
}

#[test]
fn armed_activation_should_leave_blocked_foreign_and_competing_input_untouched() -> TestResult {
    for guard in ["blocked", "unfocused", "foreign", "other-field"] {
        let session = session()?;
        let mut editor = TextCardEditor::new(&session, "a")?;
        let ctx = Context::default();
        creation_handoff(&ctx, &mut editor)?;
        if guard == "other-field" {
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("competing-field")));
        }
        let before = ctx.memory(|memory| memory.focused());
        let mut raw = RawInput {
            focused: guard != "unfocused",
            viewport_id: if guard == "foreign" {
                egui::ViewportId::from_hash_of("other")
            } else {
                egui::ViewportId::ROOT
            },
            events: vec![
                Event::Text("text".into()),
                Event::Paste("paste".into()),
                key(Key::Escape, Modifiers::NONE),
            ],
            ..Default::default()
        };
        let events = raw.events.clone();
        editor.prepare_inline_input(&ctx, &mut raw, guard == "blocked");
        assert_eq!(raw.events, events, "{guard}");
        assert_eq!(ctx.memory(|memory| memory.focused()), before, "{guard}");
        assert_eq!(editor.text(), "Original 😀");
    }
    Ok(())
}

#[test]
fn legacy_immediate_mount_should_register_real_accesskit_input_with_first_text() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::pending_insertion(&session, pending_node(&session)?)?;
    let ctx = Context::default();
    ctx.enable_accesskit();
    let (output, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![Event::Text("Immediate 日本語".into())],
        0.,
        true,
        1.,
        true,
    );
    accept_accessibility_update(&mut None, &full)?;
    assert!(output.changed && output.input.keyboard_captured);
    assert_eq!(editor.text(), "Immediate 日本語");
    Ok(())
}

#[test]
fn legacy_unarmed_mount_should_not_claim_first_frame_ime() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    ctx.enable_accesskit();
    let (output, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![Event::Ime(egui::ImeEvent::Preedit {
            text: "にほん".into(),
            active_range_chars: Some(0..3),
        })],
        0.,
        true,
        1.,
        true,
    );
    accept_accessibility_update(&mut None, &full)?;
    assert!(!output.changed && !output.input.keyboard_captured && !output.composing);
    assert_eq!(editor.text(), "Original 😀");
    assert!(ctx.memory(|memory| memory.focused()).is_none());
    Ok(())
}

#[test]
fn deferred_first_mount_should_respect_a_new_outside_pointer_action() -> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut tree = None;
    for output in creation_handoff_frames(&ctx, &mut editor, false)? {
        accept_accessibility_update(&mut tree, &output)?;
    }
    let position = Pos2::new(600., 350.);
    let (output, full) = frame_with_platform(
        &ctx,
        &mut editor,
        vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
        0.,
        true,
        1.,
        true,
    );
    accept_accessibility_update(&mut tree, &full)?;
    assert!(output.done_requested);
    assert!(!output.cancelled && !output.input.keyboard_captured);
    assert!(ctx.memory(|memory| memory.focused()).is_none());
    assert_eq!(editor.text(), "Original 😀");
    Ok(())
}
