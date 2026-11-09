//! Real egui keyboard/AccessKit paths over synthetic cards; OS acceptance is separate.
use craft_canvas_egui::*;
use craft_canvas_model::*;
use egui::{Context, Event, Id, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(
        CanvasDocument::parse(
            r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":200,"height":120,"text":"Synthetic card"},{"id":"c","type":"text","x":-230,"y":0,"width":200,"height":120,"text":"Another card"},{"id":"link","type":"link","x":240,"y":0,"width":180,"height":100,"url":"https://example.invalid"}],"edges":[{"id":"edge","fromNode":"a","toNode":"c"}]}"#,
            &limits,
        )?,
        limits,
    )
}
fn enter(modifiers: Modifiers, repeat: bool) -> Event {
    Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat,
        modifiers,
    }
}
fn selection(nodes: &[&str], edges: &[&str]) -> Selection {
    Selection {
        nodes: nodes.iter().map(|id| (*id).into()).collect(),
        edges: edges.iter().map(|id| (*id).into()).collect(),
    }
}
#[derive(Default)]
struct Dialect {
    locked: bool,
}
impl CanvasDialect for Dialect {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        let mut policy = StandardDialect.node_policy(node);
        if self.locked && node.id() == "a" {
            policy.movable = false;
        }
        policy
    }
}
#[derive(Default)]
struct Host {
    capture: bool,
    competing_field: bool,
    field_text: String,
    editor: Option<TextCardEditor>,
    inline: Option<InlineTextOutput>,
}
impl CanvasViewHost for Host {
    fn pending_text_editor(&self) -> Option<&TextCardEditor> {
        self.editor.as_ref()
    }
    fn paint_body(
        &mut self,
        ui: &mut egui::Ui,
        node: &NodeRecord,
        rect: Rect,
        zoom: f64,
    ) -> NodeBodyResponse {
        if let Some(editor) = &mut self.editor
            && editor.node_id() == node.id()
        {
            let output = editor.show_inline(ui, rect, zoom);
            let input = output.input;
            self.inline = Some(output);
            return input;
        }
        if self.competing_field && node.id() == "a" {
            let id = Id::new("competing-native-field");
            ui.memory_mut(|memory| memory.request_focus(id));
            ui.add(
                egui::TextEdit::multiline(&mut self.field_text)
                    .id(id)
                    .desired_rows(1),
            );
            // Deliberately no capture hint: the view must recheck exact current focus.
        } else {
            ui.label(node.body());
        }
        NodeBodyResponse {
            pointer_captured: false,
            keyboard_captured: self.capture,
        }
    }
}
#[derive(Default)]
struct FrameOptions {
    disabled: bool,
    noninteractive: bool,
    blocked: bool,
    unfocused: bool,
    activate_intent: bool,
}
struct NativeFrame {
    board: CanvasViewOutput,
    full: egui::FullOutput,
    remaining: Vec<Event>,
}
fn frame(
    ctx: &Context,
    view: &mut CanvasView,
    session: &CanvasSession,
    dialect: &dyn CanvasDialect,
    host: &mut Host,
    events: Vec<Event>,
    options: FrameOptions,
) -> Result<NativeFrame> {
    let mut raw = RawInput {
        events,
        focused: !options.unfocused,
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 680.))),
        ..Default::default()
    };
    if let Some(editor) = &mut host.editor {
        editor.prepare_inline_input(ctx, &mut raw, options.blocked || options.disabled);
    }
    let mut board = None;
    let mut remaining = Vec::new();
    let mut error = None;
    let mut full = ctx.run_ui(raw, |root| {
        egui::CentralPanel::default().show(root, |ui| {
            ui.add_enabled_ui(!options.disabled, |ui| {
                let mut config = CanvasViewConfig::new(Id::new("keyboard-board"), ui);
                config.interactive = !options.noninteractive && host.editor.is_none();
                config.shortcuts_blocked = options.blocked || host.editor.is_some();
                let output = view.show(ui, session, dialect, host, &config);
                if options.activate_intent {
                    for intent in &output.intents {
                        if let CanvasIntent::EditText { base, node_id } = intent {
                            assert_eq!(*base, session.identity());
                            match TextCardEditor::new(session, node_id).and_then(|mut editor| {
                                editor.begin_inline(ui.ctx())?;
                                Ok(editor)
                            }) {
                                Ok(editor) => host.editor = Some(editor),
                                Err(failure) => error = Some(failure),
                            }
                        }
                    }
                }
                remaining = ui.input(|input| input.events.clone());
                board = Some(output);
            });
        });
    });
    full.textures_delta.clear();
    full.platform_output.commands.clear();
    if let Some(error) = error {
        return Err(error);
    }
    Ok(NativeFrame {
        board: board.ok_or_else(|| CanvasError::Invalid("missing test board".into()))?,
        full,
        remaining,
    })
}
struct Changes;
impl accesskit_consumer::TreeChangeHandler for Changes {
    fn node_added(&mut self, _: &accesskit_consumer::Node<'_>) {}
    fn node_updated(&mut self, _: &accesskit_consumer::Node<'_>, _: &accesskit_consumer::Node<'_>) {
    }
    fn focus_moved(
        &mut self,
        _: Option<&accesskit_consumer::Node<'_>>,
        _: Option<&accesskit_consumer::Node<'_>>,
    ) {
    }
    fn node_removed(&mut self, _: &accesskit_consumer::Node<'_>) {}
}
fn accept_tree(tree: &mut Option<accesskit_consumer::Tree>, full: &egui::FullOutput) -> TestResult {
    let update = full
        .platform_output
        .accesskit_update
        .clone()
        .ok_or("AccessKit update")?;
    let focus = update.focus;
    let tree_id = update.tree_id;
    if let Some(tree) = tree {
        tree.update_and_process_changes(update, &mut Changes);
    } else {
        *tree = Some(accesskit_consumer::Tree::new(update, true));
    }
    assert!(
        tree.as_ref()
            .ok_or("tree")?
            .state()
            .node_by_tree_local_id(focus, tree_id)
            .is_some()
    );
    Ok(())
}

#[test]
fn enter_should_open_one_selected_text_card_through_the_accessible_native_lifecycle() -> TestResult
{
    for node_focus in [false, true] {
        for ime in [false, true] {
            let session = session()?;
            let canonical = session.document().raw_json().to_owned();
            let ctx = Context::default();
            ctx.enable_accesskit();
            let mut tree = None;
            let mut view = CanvasView::default();
            view.set_selection(selection(&["a"], &[]));
            let mut host = Host::default();
            let dialect = Dialect::default();
            let first = frame(
                &ctx,
                &mut view,
                &session,
                &dialect,
                &mut host,
                vec![],
                FrameOptions::default(),
            )?;
            accept_tree(&mut tree, &first.full)?;
            let focus = if node_focus {
                Id::new("keyboard-board").with(("node", "a"))
            } else {
                first.board.response.id
            };
            ctx.memory_mut(|memory| memory.request_focus(focus));
            let opened = frame(
                &ctx,
                &mut view,
                &session,
                &dialect,
                &mut host,
                vec![enter(Modifiers::NONE, false)],
                FrameOptions {
                    activate_intent: true,
                    ..Default::default()
                },
            )?;
            accept_tree(&mut tree, &opened.full)?;
            assert!(
                matches!(&opened.board.intents[..], [CanvasIntent::EditText { base, node_id }] if *base == session.identity() && node_id == "a")
            );
            assert!(!opened.remaining.iter().any(|event| matches!(
                event,
                Event::Key {
                    key: Key::Enter,
                    ..
                }
            )));
            assert_eq!(ctx.memory(|memory| memory.focused()), Some(focus));
            let input = if ime {
                vec![Event::Ime(egui::ImeEvent::Preedit {
                    text: "にほん".into(),
                    active_range_chars: Some(0..3),
                })]
            } else {
                vec![
                    Event::Text(" 日本語".into()),
                    Event::Paste(" e\u{301} 🪶".into()),
                ]
            };
            let written = frame(
                &ctx,
                &mut view,
                &session,
                &dialect,
                &mut host,
                input,
                FrameOptions::default(),
            )?;
            accept_tree(&mut tree, &written.full)?;
            assert!(written.board.intents.is_empty());
            assert!(
                host.inline
                    .as_ref()
                    .ok_or("inline input")?
                    .input
                    .keyboard_captured
            );
            if ime {
                assert!(
                    !written
                        .full
                        .platform_output
                        .ime
                        .ok_or("native IME")?
                        .should_interrupt_composition
                );
                let committed = frame(
                    &ctx,
                    &mut view,
                    &session,
                    &dialect,
                    &mut host,
                    vec![Event::Ime(egui::ImeEvent::Commit("日本語".into()))],
                    FrameOptions::default(),
                )?;
                accept_tree(&mut tree, &committed.full)?;
                assert_eq!(
                    host.editor.as_ref().ok_or("editor")?.text(),
                    "Synthetic card日本語"
                );
            } else {
                assert_eq!(
                    host.editor.as_ref().ok_or("editor")?.text(),
                    "Synthetic card 日本語 e\u{301} 🪶"
                );
            }
            assert_eq!(session.document().raw_json(), canonical);
            assert_eq!(session.history_usage().undo_steps, 0);
        }
    }
    Ok(())
}

#[test]
fn enter_should_remain_unconsumed_when_selection_or_input_ownership_is_ineligible() -> TestResult {
    for case in [
        "shift",
        "alt",
        "control",
        "command",
        "repeat",
        "ime",
        "locked",
        "non-text",
        "empty",
        "multiple",
        "edge",
        "disabled",
        "blocked",
        "noninteractive",
        "unfocused",
        "captured",
        "competing-field",
        "gesture",
    ] {
        let session = session()?;
        let canonical = session.document().raw_json().to_owned();
        let ctx = Context::default();
        let mut view = CanvasView::default();
        view.set_selection(selection(&["a"], &[]));
        let mut host = Host::default();
        let dialect = Dialect {
            locked: case == "locked",
        };
        let first = frame(
            &ctx,
            &mut view,
            &session,
            &dialect,
            &mut host,
            vec![],
            FrameOptions::default(),
        )?;
        ctx.memory_mut(|memory| memory.request_focus(first.board.response.id));
        match case {
            "non-text" => view.set_selection(selection(&["link"], &[])),
            "empty" => view.set_selection(selection(&[], &[])),
            "multiple" => view.set_selection(selection(&["a", "c"], &[])),
            "edge" => view.set_selection(selection(&["a"], &["edge"])),
            "captured" => host.capture = true,
            "competing-field" => {
                host.competing_field = true;
                // The node snapshot precedes paint_body transferring focus.
                ctx.memory_mut(|memory| {
                    memory.request_focus(Id::new("keyboard-board").with(("node", "a")));
                });
            }
            "gesture" => {
                let position = first
                    .board
                    .nodes
                    .iter()
                    .find(|node| node.node_id == "a")
                    .ok_or("card")?
                    .reserved
                    .center();
                frame(
                    &ctx,
                    &mut view,
                    &session,
                    &dialect,
                    &mut host,
                    vec![
                        Event::PointerMoved(position),
                        Event::PointerButton {
                            pos: position,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                    FrameOptions::default(),
                )?;
                assert!(view.has_gesture());
            }
            _ => {}
        }
        let modifiers = match case {
            "shift" => Modifiers::SHIFT,
            "alt" => Modifiers::ALT,
            "control" => Modifiers::CTRL,
            "command" => Modifiers::COMMAND,
            _ => Modifiers::NONE,
        };
        let event = enter(modifiers, case == "repeat");
        let mut events = vec![event.clone()];
        if case == "ime" {
            events.push(Event::Ime(egui::ImeEvent::Preedit {
                text: "にほん".into(),
                active_range_chars: Some(0..3),
            }));
        }
        let output = frame(
            &ctx,
            &mut view,
            &session,
            &dialect,
            &mut host,
            events,
            FrameOptions {
                disabled: case == "disabled",
                noninteractive: case == "noninteractive",
                blocked: case == "blocked",
                unfocused: case == "unfocused",
                ..Default::default()
            },
        )?;
        assert!(output.board.intents.is_empty(), "{case}");
        assert!(output.remaining.contains(&event), "{case}");
        if case == "competing-field" {
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(Id::new("competing-native-field"))
            );
            assert_eq!(host.field_text, "\n");
        }
        assert_eq!(session.document().raw_json(), canonical, "{case}");
    }
    Ok(())
}

#[test]
fn enter_inside_native_inline_text_should_insert_a_newline_without_reopening_the_card() -> TestResult
{
    let session = session()?;
    let ctx = Context::default();
    let mut view = CanvasView::default();
    view.set_selection(selection(&["a"], &[]));
    let mut host = Host::default();
    let dialect = Dialect::default();
    let first = frame(
        &ctx,
        &mut view,
        &session,
        &dialect,
        &mut host,
        vec![],
        FrameOptions::default(),
    )?;
    ctx.memory_mut(|memory| memory.request_focus(first.board.response.id));
    frame(
        &ctx,
        &mut view,
        &session,
        &dialect,
        &mut host,
        vec![enter(Modifiers::NONE, false)],
        FrameOptions {
            activate_intent: true,
            ..Default::default()
        },
    )?;
    let output = frame(
        &ctx,
        &mut view,
        &session,
        &dialect,
        &mut host,
        vec![enter(Modifiers::NONE, false)],
        FrameOptions::default(),
    )?;
    assert!(output.board.intents.is_empty());
    assert_eq!(
        host.editor.as_ref().ok_or("editor")?.text(),
        "Synthetic card\n"
    );
    let inline = host.inline.as_ref().ok_or("inline")?;
    assert!(!inline.done_requested && !inline.cancelled);
    Ok(())
}
