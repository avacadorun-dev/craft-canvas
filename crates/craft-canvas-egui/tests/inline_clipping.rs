//! Native geometry/input regressions; compiled OS input acceptance remains separate.
use craft_canvas_egui::{InlineTextOutput, TextCardEditor};
use craft_canvas_model::*;
use egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(
        CanvasDocument::parse(
            r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":300,"height":180,"text":"","future":{"keep":true}}],"edges":[]}"#,
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
fn body() -> Rect {
    Rect::from_min_size(Pos2::new(100., 100.), Vec2::new(600., 360.))
}
fn clips() -> [Rect; 4] {
    [
        Rect::from_min_max(Pos2::new(260., 100.), Pos2::new(700., 460.)),
        Rect::from_min_max(Pos2::new(100., 100.), Pos2::new(540., 460.)),
        Rect::from_min_max(Pos2::new(100., 220.), Pos2::new(700., 460.)),
        Rect::from_min_max(Pos2::new(100., 100.), Pos2::new(700., 340.)),
    ]
}
struct NativeFrame {
    inline: InlineTextOutput,
    full: egui::FullOutput,
}
struct Harness {
    ctx: Context,
    time: f64,
}
impl Harness {
    fn new() -> Self {
        Self {
            ctx: Context::default(),
            time: 0.,
        }
    }
    fn frame(
        &mut self,
        editor: &mut TextCardEditor,
        clip: Rect,
        events: Vec<Event>,
    ) -> NativeFrame {
        self.time += 0.1;
        let mut raw = RawInput {
            events,
            time: Some(self.time),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 680.))),
            ..Default::default()
        };
        editor.prepare_inline_input(&self.ctx, &mut raw, false);
        let mut inline = InlineTextOutput::default();
        let mut full = self.ctx.run_ui(raw, |root| {
            egui::CentralPanel::default().show(root, |ui| {
                ui.scope_builder(
                    egui::UiBuilder::new().id_salt("card-body").max_rect(body()),
                    |ui| {
                        ui.set_clip_rect(clip);
                        inline = editor.show_inline(ui, body(), 2.);
                    },
                );
            });
        });
        full.textures_delta.clear();
        full.platform_output.commands.clear();
        NativeFrame { inline, full }
    }
    // ScrollArea requests a repaint after applying its native caret target. These
    // are ordinary redraws after input; the first input frame is never skipped.
    fn settle(&mut self, editor: &mut TextCardEditor, clip: Rect) -> NativeFrame {
        for _ in 0..5 {
            self.frame(editor, clip, vec![]);
        }
        self.frame(editor, clip, vec![])
    }
}
fn visible_caret(frame: &NativeFrame, clip: Rect) -> TestResult {
    let ime = frame
        .full
        .platform_output
        .ime
        .as_ref()
        .ok_or("native IME geometry")?;
    assert!(
        clip.contains_rect(ime.cursor_rect),
        "caret {:?} outside {:?}",
        ime.cursor_rect,
        clip
    );
    assert!(!ime.should_interrupt_composition);
    assert!(
        frame.full.shapes.iter().any(|shape| {
            matches!(shape.shape, egui::Shape::Text(_))
                && shape.clip_rect.intersects(ime.cursor_rect)
        }),
        "native caret must intersect the actual text paint clip"
    );
    assert!(frame.inline.input.keyboard_captured);
    assert!(!frame.inline.cancelled && !frame.inline.done_requested);
    Ok(())
}
fn layout(
    frame: &NativeFrame,
    text: &str,
) -> std::result::Result<(u32, Vec<usize>), Box<dyn std::error::Error>> {
    let galley = frame
        .full
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(shape) if shape.galley.text() == text => Some(&shape.galley),
            _ => None,
        })
        .ok_or("actual painted text galley")?;
    Ok((
        galley.job.wrap.max_width.to_bits(),
        galley.rows.iter().map(|row| row.glyphs.len()).collect(),
    ))
}

#[test]
fn clipping_each_edge_should_reveal_native_caret_without_reflowing_or_changing_the_card()
-> TestResult {
    for text in [
        "M".repeat(32),
        "A wrapped sentence keeps the original card width. ".repeat(16),
    ] {
        for clip in clips() {
            for start_clipped in [false, true] {
                let session = session()?;
                let canonical = session.document().raw_json().to_owned();
                let mut editor = TextCardEditor::new(&session, "a")?;
                let mut native = Harness::new();
                let initial_clip = if start_clipped { clip } else { body() };
                native.frame(&mut editor, initial_clip, vec![]);
                if start_clipped {
                    // Reproduce the reported order: clipping is already stable before input.
                    native.settle(&mut editor, initial_clip);
                }
                let input =
                    native.frame(&mut editor, initial_clip, vec![Event::Text(text.clone())]);
                assert!(input.inline.changed);
                assert_eq!(editor.text(), text);
                visible_caret(&native.settle(&mut editor, initial_clip), initial_clip)?;
                let baseline = native.settle(&mut editor, body());
                let original_layout = layout(&baseline, &text)?;
                let identity = native.ctx.memory(|memory| memory.focused());
                let clipped = native.settle(&mut editor, clip);
                visible_caret(&clipped, clip)?;
                assert_eq!(layout(&clipped, &text)?, original_layout);
                assert_eq!(native.ctx.memory(|memory| memory.focused()), identity);

                // Move to both text ends through native keyboard selection, not state injection.
                for key_code in [Key::ArrowUp, Key::ArrowDown] {
                    native.frame(
                        &mut editor,
                        clip,
                        vec![key(
                            key_code,
                            Modifiers {
                                command: true,
                                shift: true,
                                ..Modifiers::NONE
                            },
                        )],
                    );
                    let selected = native.settle(&mut editor, clip);
                    visible_caret(&selected, clip)?;
                    assert_eq!(layout(&selected, &text)?, original_layout);
                    let state =
                        egui::TextEdit::load_state(&native.ctx, identity.ok_or("editor focus")?)
                            .ok_or("native edit state")?;
                    let range = state.cursor.char_range().ok_or("native selection")?;
                    let count = text.chars().count();
                    assert_eq!(range.secondary.index.0, count);
                    assert_eq!(
                        range.primary.index.0,
                        if key_code == Key::ArrowUp { 0 } else { count }
                    );
                }
                assert_eq!(editor.text(), text);
                assert_eq!(session.document().raw_json(), canonical);
                assert_eq!(session.history_usage().undo_steps, 0);
            }
        }
    }
    Ok(())
}

#[test]
fn composition_and_local_undo_should_keep_the_same_native_editor_across_clipping() -> TestResult {
    for clip in clips() {
        let session = session()?;
        let mut editor = TextCardEditor::new(&session, "a")?;
        let mut native = Harness::new();
        native.frame(&mut editor, body(), vec![]);
        let text = "A wrapped line for a retained composition. ".repeat(12);
        native.frame(&mut editor, body(), vec![Event::Paste(text.clone())]);
        native.settle(&mut editor, body());
        let identity = native.ctx.memory(|memory| memory.focused());
        let preedit = native.frame(
            &mut editor,
            clip,
            vec![Event::Ime(egui::ImeEvent::Preedit {
                text: "nihon".into(),
                active_range_chars: Some(0..5),
            })],
        );
        assert!(preedit.inline.composing);
        assert!(
            !preedit
                .full
                .platform_output
                .ime
                .as_ref()
                .ok_or("first preedit IME")?
                .should_interrupt_composition
        );
        let settled = native.settle(&mut editor, clip);
        visible_caret(&settled, clip)?;
        assert!(settled.inline.composing);
        let committed = native.frame(
            &mut editor,
            clip,
            vec![Event::Ime(egui::ImeEvent::Commit("日本語".into()))],
        );
        assert!(!committed.inline.cancelled && !committed.inline.done_requested);
        assert_eq!(editor.text(), format!("{text}日本語"));
        visible_caret(&native.settle(&mut editor, clip), clip)?;
        assert_eq!(native.ctx.memory(|memory| memory.focused()), identity);

        // Let the native Undoer create its ordinary one-second stable checkpoint.
        native.time += 1.1;
        native.frame(&mut editor, clip, vec![]);
        // A separate native paste/Undo still uses the retained local history.
        native.frame(&mut editor, clip, vec![Event::Paste(" later text".into())]);
        native.settle(&mut editor, clip);
        native.frame(&mut editor, clip, vec![key(Key::Z, Modifiers::COMMAND)]);
        assert_eq!(editor.text(), format!("{text}日本語"));
        visible_caret(&native.settle(&mut editor, clip), clip)?;
    }
    Ok(())
}

#[test]
fn unchanged_clipping_should_allow_manual_scroll_and_empty_clipping_should_not_accept_input()
-> TestResult {
    let session = session()?;
    let mut editor = TextCardEditor::new(&session, "a")?;
    let mut native = Harness::new();
    let clip = clips()[3];
    native.frame(&mut editor, clip, vec![]);
    let text = "A long note with enough lines to scroll independently. ".repeat(32);
    native.frame(&mut editor, clip, vec![Event::Paste(text.clone())]);
    let before = native.settle(&mut editor, clip);
    visible_caret(&before, clip)?;
    let before_y = before
        .full
        .platform_output
        .ime
        .as_ref()
        .ok_or("before IME")?
        .cursor_rect
        .min
        .y;
    native.frame(
        &mut editor,
        clip,
        vec![
            Event::PointerMoved(clip.center()),
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0., 240.),
                phase: egui::TouchPhase::Move,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let after = native.settle(&mut editor, clip);
    let after_y = after
        .full
        .platform_output
        .ime
        .as_ref()
        .ok_or("after IME")?
        .cursor_rect
        .min
        .y;
    assert!(after_y > before_y + 30., "manual scroll was overridden");
    let stationary = native.settle(&mut editor, clip);
    let stationary_y = stationary
        .full
        .platform_output
        .ime
        .as_ref()
        .ok_or("stationary IME")?
        .cursor_rect
        .min
        .y;
    assert!((stationary_y - after_y).abs() < 1.);
    let hidden = native.frame(
        &mut editor,
        Rect::NOTHING,
        vec![Event::Text("must not appear".into())],
    );
    assert!(!hidden.inline.changed && !hidden.inline.input.keyboard_captured);
    assert!(hidden.full.platform_output.ime.is_none());
    assert_eq!(editor.text(), text);
    Ok(())
}
