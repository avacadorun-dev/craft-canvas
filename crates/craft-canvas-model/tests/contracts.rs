//! Synthetic semantic contracts; no private project data.
use craft_canvas_model::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const BOARD: &str = r#"
 {"future":{"precise":123456789012345678901234567890.12345678900,"huge":1e9999},"nodes":[
 { "id":"a", "type":"text", "x":0,"y":0,"width":200,"height":100,"text":"Привет 🌙 日本語", "future":{"keep":1.23000,"nested":{"v":900719925474099312345}} },
 {"id":"b","type":"link","x":300,"y":0,"width":200,"height":100,"url":"https://example.invalid/inert","extra":[3,2,1]},
 {"id":"g","type":"group","x":-40,"y":-40,"width":600,"height":200,"label":"Group","opaque":"keep"}
 ],"edges":[{ "id":"e","fromNode":"a","toNode":"b","fromSide":"right","toSide":"left","label":"next","future":{"v":2.3400}}]}
 "#;
fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(CanvasDocument::parse(BOARD, &limits)?, limits)
}
fn transaction(session: &CanvasSession, commands: Vec<CanvasCommand>) -> CanvasTransaction {
    CanvasTransaction::new(session.identity(), commands)
}
fn edit(session: &mut CanvasSession, text: &str) -> Result<()> {
    let expected = session
        .document()
        .node("a")
        .ok_or_else(|| CanvasError::NotFound("a".into()))?
        .body()
        .to_owned();
    let tx = transaction(
        session,
        vec![CanvasCommand::EditText {
            node_id: "a".into(),
            expected,
            text: text.into(),
        }],
    );
    let change = session.prepare(tx, &StandardDialect)?;
    session.apply(change)?;
    Ok(())
}
#[test]
fn no_edit_export_retains_every_original_byte_and_future_number() -> TestResult {
    let session = session()?;
    assert_eq!(session.document().raw_json(), BOARD);
    assert_eq!(
        session
            .document()
            .nodes()
            .iter()
            .map(|n| n.id())
            .collect::<Vec<_>>(),
        ["a", "b", "g"]
    );
    assert!(
        session
            .document()
            .field("future")
            .ok_or("future")?
            .get()
            .contains("1e9999")
    );
    Ok(())
}
#[test]
fn known_text_edit_preserves_unrelated_raw_records_unknown_values_and_one_undo() -> TestResult {
    let mut session = session()?;
    let untouched = session
        .document()
        .node("b")
        .ok_or("b")?
        .raw_json()
        .to_owned();
    let future = session
        .document()
        .node("a")
        .ok_or("a")?
        .field("future")
        .ok_or("future")?
        .get()
        .to_owned();
    edit(&mut session, "Новое e\u{301} 🪶")?;
    assert_eq!(
        session
            .document()
            .node("a")
            .ok_or("a")?
            .field("future")
            .ok_or("future")?
            .get(),
        future
    );
    assert_eq!(
        session.document().node("b").ok_or("b")?.raw_json(),
        untouched
    );
    assert_eq!(session.history_usage().undo_steps, 1);
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), BOARD);
    let redo = session.prepare_redo()?;
    session.apply(redo)?;
    assert_eq!(
        session.document().node("a").ok_or("a")?.body(),
        "Новое e\u{301} 🪶"
    );
    assert_eq!(session.identity().revision(), 3);
    Ok(())
}
#[test]
fn duplicate_keys_are_refused_even_when_escaped_or_inside_unknown_envelopes() {
    let limits = CanvasLimits::default();
    for raw in [
        r#"{"nodes":[],"edges":[],"future":{"x":1,"\u0078":2}}"#,
        r#"{"nodes":[],"\u006eodes":[],"edges":[]}"#,
        r#"{"nodes":[{"id":"x","type":"text","x":0,"x":1,"y":0,"width":40,"height":40,"text":""}],"edges":[]}"#,
    ] {
        assert!(matches!(
            CanvasDocument::parse(raw, &limits),
            Err(CanvasError::DuplicateKey(_))
        ));
    }
}
#[test]
fn array_count_and_nesting_are_refused_before_graph_allocation() {
    let limits = CanvasLimits {
        max_nodes: 1,
        max_depth: 4,
        ..CanvasLimits::default()
    };
    assert!(matches!(
        CanvasDocument::parse(r#"{"nodes":[{},{}],"edges":[]}"#, &limits),
        Err(CanvasError::Limit(_))
    ));
    assert!(matches!(
        CanvasDocument::parse(r#"{"nodes":[],"edges":[],"opaque":[[[[[0]]]]]}"#, &limits),
        Err(CanvasError::Limit(_))
    ));
}
#[test]
fn malformed_json_and_invalid_geometry_never_become_a_document() {
    let limits = CanvasLimits::default();
    for raw in [
        "{} trailing",
        "[",
        r#"{"nodes":[],"edges":[],"s":"bad\q"}"#,
        r#"{"nodes":[{"id":"x","type":"text","x":0.5,"y":0,"width":40,"height":40,"text":""}],"edges":[]}"#,
        r#"{"nodes":[{"id":"x","type":"text","x":10000001,"y":0,"width":40,"height":40,"text":""}],"edges":[]}"#,
    ] {
        assert!(
            CanvasDocument::parse(raw, &limits).is_err(),
            "accepted {raw}"
        );
    }
}
#[test]
fn unknown_endpoints_and_duplicate_record_ids_are_refused() {
    let limits = CanvasLimits::default();
    for raw in [
        r#"{"nodes":[],"edges":[{"id":"e","fromNode":"a","toNode":"a"}]}"#,
        r#"{"nodes":[{"id":"x","type":"text","x":0,"y":0,"width":40,"height":40,"text":""},{"id":"x","type":"text","x":0,"y":0,"width":40,"height":40,"text":""}],"edges":[]}"#,
    ] {
        assert!(CanvasDocument::parse(raw, &limits).is_err());
    }
}
#[test]
fn a_late_or_foreign_prepared_change_cannot_mutate_newer_content() -> TestResult {
    let mut first = session()?;
    let mut foreign = session()?;
    let pending = first.prepare(
        transaction(
            &first,
            vec![CanvasCommand::EditText {
                node_id: "a".into(),
                expected: "Привет 🌙 日本語".into(),
                text: "late".into(),
            }],
        ),
        &StandardDialect,
    )?;
    let other = first.prepare(transaction(&first, vec![]), &StandardDialect)?;
    assert!(matches!(foreign.apply(other), Err(CanvasError::StaleBase)));
    edit(&mut first, "newer")?;
    let current = first.document().raw_json().to_owned();
    assert!(matches!(first.apply(pending), Err(CanvasError::StaleBase)));
    assert_eq!(first.document().raw_json(), current);
    assert_eq!(first.history_usage().undo_steps, 1);
    Ok(())
}
#[test]
fn invalid_second_command_rolls_back_the_complete_candidate() -> TestResult {
    let first = session()?;
    let tx = transaction(
        &first,
        vec![
            CanvasCommand::EditText {
                node_id: "a".into(),
                expected: "Привет 🌙 日本語".into(),
                text: "not committed".into(),
            },
            CanvasCommand::SetGeometry {
                node_id: "missing".into(),
                expected: WorldRect {
                    x: 0.,
                    y: 0.,
                    width: 40.,
                    height: 40.,
                },
                replacement: WorldRect {
                    x: 100.,
                    y: 0.,
                    width: 40.,
                    height: 40.,
                },
            },
        ],
    );
    assert!(first.prepare(tx, &StandardDialect).is_err());
    assert_eq!(first.document().raw_json(), BOARD);
    assert_eq!(first.history_usage().undo_steps, 0);
    Ok(())
}
#[test]
fn spatial_group_move_includes_selected_members_only_once_and_undo_is_exact() -> TestResult {
    let mut first = session()?;
    let tx = transaction(
        &first,
        vec![CanvasCommand::Translate {
            node_ids: vec!["g".into(), "a".into()],
            delta: WorldPoint { x: 20., y: 10. },
        }],
    );
    let change = first.prepare(tx, &StandardDialect)?;
    assert_eq!(change.document().node("a").ok_or("a")?.bounds().x, 20.);
    assert_eq!(change.document().node("b").ok_or("b")?.bounds().x, 320.);
    assert_eq!(change.document().node("g").ok_or("g")?.bounds().x, -20.);
    first.apply(change)?;
    let undo = first.prepare_undo()?;
    first.apply(undo)?;
    assert_eq!(first.document().raw_json(), BOARD);
    Ok(())
}
struct Frames;
impl CanvasDialect for Frames {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        NodePolicy {
            movable: node.id() != "b",
            resizable: true,
            group: if node.id() == "g" {
                GroupBehavior::Frame
            } else {
                GroupBehavior::Independent
            },
        }
    }
}
#[test]
fn frame_move_does_not_move_enclosed_cards_and_lock_refusal_is_atomic() -> TestResult {
    let first = session()?;
    let change = first.prepare(
        transaction(
            &first,
            vec![CanvasCommand::Translate {
                node_ids: vec!["g".into()],
                delta: WorldPoint { x: 20., y: 0. },
            }],
        ),
        &Frames,
    )?;
    assert_eq!(change.document().node("a").ok_or("a")?.bounds().x, 0.);
    assert!(matches!(
        first.prepare(
            transaction(
                &first,
                vec![CanvasCommand::Translate {
                    node_ids: vec!["b".into()],
                    delta: WorldPoint { x: 20., y: 0. }
                }]
            ),
            &Frames
        ),
        Err(CanvasError::NotEditable(_))
    ));
    assert_eq!(first.document().raw_json(), BOARD);
    Ok(())
}
#[test]
fn deleting_a_group_retains_members_and_deleting_card_removes_incident_edge() -> TestResult {
    let mut first = session()?;
    let group = Selection {
        nodes: BTreeSet::from(["g".into()]),
        edges: BTreeSet::new(),
    };
    let change = first.prepare(
        transaction(&first, vec![CanvasCommand::RemoveSelection(group)]),
        &StandardDialect,
    )?;
    first.apply(change)?;
    assert_eq!(first.document().nodes().len(), 2);
    assert_eq!(first.document().edges().len(), 1);
    let card = Selection {
        nodes: BTreeSet::from(["a".into()]),
        edges: BTreeSet::new(),
    };
    let change = first.prepare(
        transaction(&first, vec![CanvasCommand::RemoveSelection(card)]),
        &StandardDialect,
    )?;
    assert!(change.selection_map().removed_edges.contains("e"));
    first.apply(change)?;
    assert!(first.document().edges().is_empty());
    let undo = first.prepare_undo()?;
    assert!(undo.selection_map().added_edges.contains("e"));
    first.apply(undo)?;
    assert!(first.document().edge("e").is_some());
    Ok(())
}
#[test]
fn duplicate_uses_complete_fresh_ids_internal_edges_and_preserved_unknown_fields() -> TestResult {
    let first = session()?;
    let ids = IdMapping {
        nodes: BTreeMap::from([
            ("g".into(), "g2".into()),
            ("a".into(), "a2".into()),
            ("b".into(), "b2".into()),
        ]),
        edges: BTreeMap::from([("e".into(), "e2".into())]),
    };
    let change = first.prepare(
        transaction(
            &first,
            vec![CanvasCommand::Duplicate {
                node_ids: vec!["g".into()],
                ids,
                delta: WorldPoint { x: 700., y: 0. },
            }],
        ),
        &StandardDialect,
    )?;
    assert_eq!(change.document().edge("e2").ok_or("e2")?.from_node(), "a2");
    assert_eq!(change.document().node("a2").ok_or("a2")?.bounds().x, 700.);
    assert_eq!(
        change
            .document()
            .node("a2")
            .ok_or("a2")?
            .field("future")
            .ok_or("future")?
            .get(),
        first
            .document()
            .node("a")
            .ok_or("a")?
            .field("future")
            .ok_or("future")?
            .get()
    );
    assert_eq!(change.selection_map().added_nodes.len(), 3);
    assert_eq!(first.document().nodes().len(), 3);
    Ok(())
}
#[test]
fn incomplete_duplicate_id_allocation_never_mutates_graph() -> TestResult {
    let first = session()?;
    let result = first.prepare(
        transaction(
            &first,
            vec![CanvasCommand::Duplicate {
                node_ids: vec!["g".into()],
                ids: IdMapping::default(),
                delta: WorldPoint { x: 1., y: 1. },
            }],
        ),
        &StandardDialect,
    );
    assert!(result.is_err());
    assert_eq!(first.document().raw_json(), BOARD);
    Ok(())
}
#[test]
fn nested_field_patch_retains_unrelated_large_numeric_lexemes() -> TestResult {
    let mut first = session()?;
    let update = FieldUpdate::set(
        vec!["future".into(), "nested".into(), "new".into()],
        r#"{"safe":true}"#,
        first.limits(),
    )?;
    let change = first.prepare(
        transaction(
            &first,
            vec![CanvasCommand::SetNodeFields {
                node_id: "a".into(),
                fields: vec![update],
            }],
        ),
        &StandardDialect,
    )?;
    first.apply(change)?;
    let future = first
        .document()
        .node("a")
        .ok_or("a")?
        .field("future")
        .ok_or("future")?
        .get();
    assert!(future.contains("900719925474099312345"));
    assert!(future.contains("1.23000"));
    assert!(future.contains("\"new\":{\"safe\":true}"));
    let undo = first.prepare_undo()?;
    first.apply(undo)?;
    assert_eq!(first.document().raw_json(), BOARD);
    Ok(())
}
#[test]
fn complete_reorder_keeps_records_exact_and_partial_permutations_fail() -> TestResult {
    let first = session()?;
    let exact = first.document().node("a").ok_or("a")?.raw_json().to_owned();
    let change = first.prepare(
        transaction(
            &first,
            vec![CanvasCommand::ReorderNodes {
                order: vec!["g".into(), "b".into(), "a".into()],
            }],
        ),
        &StandardDialect,
    )?;
    assert_eq!(change.document().nodes()[2].raw_json(), exact);
    assert!(
        first
            .prepare(
                transaction(
                    &first,
                    vec![CanvasCommand::ReorderNodes {
                        order: vec!["a".into()]
                    }]
                ),
                &StandardDialect
            )
            .is_err()
    );
    Ok(())
}
#[test]
fn no_op_never_creates_history_and_new_edit_discards_redo() -> TestResult {
    let mut first = session()?;
    edit(&mut first, "Привет 🌙 日本語")?;
    assert_eq!(first.identity().revision(), 0);
    assert_eq!(first.history_usage().undo_steps, 0);
    edit(&mut first, "one")?;
    let undo = first.prepare_undo()?;
    first.apply(undo)?;
    assert_eq!(first.history_usage().redo_steps, 1);
    edit(&mut first, "two")?;
    assert_eq!(first.history_usage().redo_steps, 0);
    assert!(matches!(first.prepare_redo(), Err(CanvasError::NoHistory)));
    Ok(())
}
#[test]
fn complete_history_steps_are_pruned_and_budget_refusal_precedes_mutation() -> TestResult {
    let limits = CanvasLimits {
        max_history_steps: 2,
        ..CanvasLimits::default()
    };
    let mut first = CanvasSession::new(CanvasDocument::parse(BOARD, &limits)?, limits)?;
    edit(&mut first, "one")?;
    edit(&mut first, "two")?;
    edit(&mut first, "three")?;
    assert_eq!(first.history_usage().undo_steps, 2);
    for _ in 0..2 {
        let undo = first.prepare_undo()?;
        first.apply(undo)?;
    }
    assert_eq!(first.document().node("a").ok_or("a")?.body(), "one");
    assert!(matches!(first.prepare_undo(), Err(CanvasError::NoHistory)));
    let limits = CanvasLimits {
        max_history_bytes: 1,
        ..CanvasLimits::default()
    };
    let mut tight = CanvasSession::new(CanvasDocument::parse(BOARD, &limits)?, limits)?;
    assert!(matches!(
        edit(&mut tight, "larger"),
        Err(CanvasError::HistoryBudget)
    ));
    assert_eq!(tight.document().raw_json(), BOARD);
    assert_eq!(tight.identity().revision(), 0);
    Ok(())
}
#[test]
fn work_exhaustion_and_cancellation_leave_canonical_history_untouched() -> TestResult {
    let first = session()?;
    let cancelled = AtomicBool::new(true);
    let mut control = WorkControl::new(1_000, Some(&cancelled));
    assert!(matches!(
        first.prepare_with_control(
            transaction(
                &first,
                vec![CanvasCommand::Translate {
                    node_ids: vec!["a".into()],
                    delta: WorldPoint { x: 1., y: 1. }
                }]
            ),
            &StandardDialect,
            &mut control
        ),
        Err(CanvasError::Cancelled)
    ));
    let mut control = WorkControl::new(0, None);
    assert!(matches!(
        first.prepare_with_control(transaction(&first, vec![]), &StandardDialect, &mut control),
        Err(CanvasError::WorkLimit)
    ));
    assert_eq!(first.document().raw_json(), BOARD);
    assert_eq!(first.history_usage().undo_steps, 0);
    Ok(())
}
#[test]
fn camera_roundtrip_and_pointer_anchor_preserve_precision_at_world_extremes() -> TestResult {
    for world in [-10_000_000., 10_000_000.] {
        for zoom in [Camera::MIN_ZOOM, 1., Camera::MAX_ZOOM] {
            let mut camera = Camera::new(WorldPoint { x: world, y: world }, zoom)?;
            let size = WorldPoint { x: 1440., y: 940. };
            let p = WorldPoint {
                x: world + 0.125,
                y: world + 0.25,
            };
            let local = camera.world_to_view(p, size);
            let actual = camera.view_to_world(local, size);
            assert!((actual.x - p.x).abs() < 0.000001);
            assert!((actual.y - p.y).abs() < 0.000001);
            let pointer = WorldPoint { x: 430., y: 270. };
            let before = camera.view_to_world(pointer, size);
            camera.zoom_at(1.5, pointer, size)?;
            let after = camera.view_to_world(pointer, size);
            assert!((after.x - before.x).abs() < 0.000001);
            assert!((after.y - before.y).abs() < 0.000001);
        }
    }
    Ok(())
}
#[test]
fn invalid_camera_changes_are_atomic_and_fit_respects_zoom_range() -> TestResult {
    let mut camera = Camera::default();
    let before = camera;
    assert!(
        camera
            .zoom_at(
                f64::NAN,
                WorldPoint::default(),
                WorldPoint { x: 1000., y: 680. }
            )
            .is_err()
    );
    assert_eq!(camera, before);
    assert!(
        camera
            .pan(WorldPoint {
                x: f64::INFINITY,
                y: 0.
            })
            .is_err()
    );
    assert_eq!(camera, before);
    camera.fit(
        WorldRect {
            x: 0.,
            y: 0.,
            width: 100.,
            height: 100.,
        },
        WorldPoint { x: 1000., y: 680. },
        24.,
    )?;
    assert_eq!(camera.zoom(), Camera::MAX_ZOOM);
    Ok(())
}

#[test]
fn history_budget_keeps_source_capacity_separate_from_current_parsed_cache() -> TestResult {
    let limits = CanvasLimits {
        max_bytes: 2048,
        max_history_bytes: 4096,
        ..CanvasLimits::default()
    };
    let raw = serde_json::json!({"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":200,"height":120,"text":"A".repeat(100),"future":{"opaque":"X".repeat(1000)}}],"edges":[]}).to_string();
    let source_size = raw.len();
    let limits = CanvasLimits {
        max_history_bytes: source_size * 2,
        ..limits
    };
    let mut session = CanvasSession::new(CanvasDocument::parse(&raw, &limits)?, limits)?;
    for character in ['B', 'C', 'D'] {
        let expected = session.document().node("a").ok_or("a")?.body().to_owned();
        let command = CanvasCommand::EditText {
            node_id: "a".into(),
            expected,
            text: character.to_string().repeat(100),
        };
        let change = session.prepare(
            CanvasTransaction::new(session.identity(), vec![command]),
            &StandardDialect,
        )?;
        session.apply(change)?;
    }
    assert_eq!(session.history_usage().undo_steps, 2);
    assert!(
        session.history_usage().current_payload_bytes
            > session.history_usage().current_source_bytes
    );
    assert!(session.history_usage().history_budget_bytes <= source_size * 2);
    assert!(session.history_usage().retained_source_bytes <= source_size * 3);
    for _ in 0..2 {
        let change = session.prepare_undo()?;
        session.apply(change)?;
    }
    assert_eq!(
        session.document().node("a").ok_or("a")?.body(),
        "B".repeat(100)
    );
    for _ in 0..2 {
        let change = session.prepare_redo()?;
        session.apply(change)?;
    }
    assert_eq!(
        session.document().node("a").ok_or("a")?.body(),
        "D".repeat(100)
    );
    Ok(())
}
#[test]
fn full_source_node_capacity_can_move_and_undo_in_one_charged_operation() -> TestResult {
    let limits = CanvasLimits::default();
    let nodes:Vec<_>=(0..limits.max_nodes).map(|i|serde_json::json!({"id":format!("n{i}"),"type":"text","x":(i%100)*100,"y":(i/100)*100,"width":80,"height":80,"text":"synthetic"})).collect();
    let raw = serde_json::json!({"nodes":nodes,"edges":[]}).to_string();
    let mut session = CanvasSession::new(CanvasDocument::parse(&raw, &limits)?, limits)?;
    let node_ids = session
        .document()
        .nodes()
        .iter()
        .map(|node| node.id().to_owned())
        .collect();
    let change = session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::Translate {
                node_ids,
                delta: WorldPoint { x: 1., y: 1. },
            }],
        ),
        &StandardDialect,
    )?;
    session.apply(change)?;
    assert_eq!(session.document().nodes().len(), 5000);
    assert_eq!(
        session.document().node("n4999").ok_or("last")?.bounds().x,
        9901.
    );
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), raw);
    Ok(())
}

#[test]
fn source_compatible_long_url_and_absent_optional_canvas_fields_remain_editable() -> TestResult {
    let limits = CanvasLimits::default();
    let url = format!("https://example.invalid/{}", "a".repeat(1_500));
    let raw = serde_json::json!({"nodes":[{"id":"legacy-link","type":"link","x":0,"y":0,"width":300,"height":180,"url":url,"future":{"keep":true}}],"edges":[]}).to_string();
    let mut session = CanvasSession::new(CanvasDocument::parse(&raw, &limits)?, limits)?;
    let original = session
        .document()
        .node("legacy-link")
        .ok_or("legacy link")?
        .raw_json()
        .to_owned();
    let change = session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::Translate {
                node_ids: vec!["legacy-link".into()],
                delta: WorldPoint { x: 20.0, y: 0.0 },
            }],
        ),
        &StandardDialect,
    )?;
    session.apply(change)?;
    let moved = session
        .document()
        .node("legacy-link")
        .ok_or("legacy link")?;
    assert_eq!(moved.body(), url);
    assert_eq!(moved.bounds().x, 20.0);
    assert!(moved.field("x-writers-studio").is_none());
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(
        session
            .document()
            .node("legacy-link")
            .ok_or("legacy link")?
            .raw_json(),
        original
    );
    Ok(())
}

#[test]
fn url_byte_bound_is_distinct_from_file_character_bound() -> TestResult {
    let limits = CanvasLimits::default();
    let url = format!("https://example.invalid/{}", "界".repeat(700));
    let too_long = serde_json::json!({"id":"link","type":"link","x":0,"y":0,"width":300,"height":180,"url":url}).to_string();
    assert!(NodeRecord::parse(&too_long, &limits).is_err());
    let file = serde_json::json!({"id":"file","type":"file","x":0,"y":0,"width":300,"height":180,"file":"界".repeat(1_024)}).to_string();
    assert!(NodeRecord::parse(&file, &limits).is_ok());
    Ok(())
}
