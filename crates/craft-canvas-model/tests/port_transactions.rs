//! Atomic named-port mutations, preservation and final candidate regressions.
use craft_canvas_model::*;
use std::collections::BTreeMap;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const BOARD: &str = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"a","craftCanvas":{"version":1,"ports":[{"id":"out","kind":"source","side":"right","x":100,"y":25},{"id":"aux","kind":"source","side":"right","x":100,"y":75}],"future":1.23000}},{"id":"b","type":"text","x":200,"y":0,"width":100,"height":100,"text":"b","craftCanvas":{"version":1,"ports":[{"id":"in","kind":"target","side":"left","x":0,"y":50}]}},{"id":"c","type":"text","x":200,"y":200,"width":100,"height":100,"text":"c","craftCanvas":{"version":1,"ports":[{"id":"in","kind":"target","side":"left","x":0,"y":50}]}}],"edges":[{"id":"e","fromNode":"a","toNode":"b","toSide":null,"craftCanvas":{"version":1,"fromPort":{"id":"out","kind":"source"},"toPort":{"id":"in","kind":"target"},"future":{"n":1.23000}}}]}"#;

fn session(raw: &str) -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(CanvasDocument::parse(raw, &limits)?, limits)
}
fn endpoint(node: &str, id: &str, kind: PortKind) -> PortEndpoint {
    PortEndpoint {
        node_id: node.into(),
        binding: PortBinding::Named(PortKey {
            id: id.into(),
            kind,
        }),
    }
}
fn reconnect_from() -> CanvasCommand {
    CanvasCommand::ReconnectPort {
        edge_id: "e".into(),
        endpoint: ConnectorEndpointKind::From,
        expected: endpoint("a", "out", PortKind::Source),
        replacement: endpoint("a", "aux", PortKind::Source),
    }
}
fn retain_aux(session: &CanvasSession) -> TestResultCommand {
    let ports = session
        .document()
        .node("a")
        .ok_or("a")?
        .ports()
        .iter()
        .filter(|port| port.key.id == "aux")
        .cloned()
        .collect();
    Ok(CanvasCommand::SetPorts {
        node_id: "a".into(),
        ports,
    })
}
type TestResultCommand = std::result::Result<CanvasCommand, Box<dyn std::error::Error>>;

#[test]
fn port_removal_refuses_or_repairs_against_final_candidate_in_either_order() -> TestResult {
    let initial = session(BOARD)?;
    assert!(
        initial
            .prepare(
                CanvasTransaction::new(initial.identity(), vec![retain_aux(&initial)?]),
                &StandardDialect
            )
            .is_err()
    );
    assert_eq!(initial.document().raw_json(), BOARD);
    assert_eq!(initial.history_usage().undo_steps, 0);
    for removal_first in [true, false] {
        let mut state = session(BOARD)?;
        let mut commands = vec![retain_aux(&state)?, reconnect_from()];
        if !removal_first {
            commands.reverse();
        }
        let prepared = state.prepare(
            CanvasTransaction::new(state.identity(), commands),
            &StandardDialect,
        )?;
        state.apply(prepared)?;
        assert_eq!(
            state
                .document()
                .edge("e")
                .ok_or("e")?
                .endpoint(ConnectorEndpointKind::From)?,
            endpoint("a", "aux", PortKind::Source)
        );
        assert_eq!(state.document().node("a").ok_or("a")?.ports().len(), 1);
        assert_eq!(
            state
                .document()
                .edge("e")
                .ok_or("e")?
                .field("toSide")
                .ok_or("side")?
                .get(),
            "null"
        );
        let saved = state.document().raw_json().to_owned();
        let reopened = CanvasDocument::parse(&saved, state.limits())?;
        assert_eq!(
            reopened
                .edge("e")
                .ok_or("e")?
                .endpoint(ConnectorEndpointKind::From)?,
            endpoint("a", "aux", PortKind::Source)
        );
        state.apply(state.prepare_undo()?)?;
        assert_eq!(state.document().raw_json(), BOARD);
        state.apply(state.prepare_redo()?)?;
        assert_eq!(state.document().raw_json(), saved);
    }
    Ok(())
}

#[test]
fn raw_endpoint_and_catalog_edits_cannot_bypass_named_validation() -> TestResult {
    for (node_edit, path, value) in [
        (false, vec!["craftCanvas", "toPort", "id"], r#""missing""#),
        (true, vec!["craftCanvas", "ports"], "[]"),
    ] {
        let state = session(BOARD)?;
        let fields = vec![FieldUpdate::set(
            path.into_iter().map(str::to_owned).collect(),
            value,
            state.limits(),
        )?];
        let command = if node_edit {
            CanvasCommand::SetNodeFields {
                node_id: "a".into(),
                fields,
            }
        } else {
            CanvasCommand::SetEdgeFields {
                edge_id: "e".into(),
                fields,
            }
        };
        assert!(
            state
                .prepare(
                    CanvasTransaction::new(state.identity(), vec![command]),
                    &StandardDialect
                )
                .is_err()
        );
        assert_eq!(state.document().raw_json(), BOARD);
        assert_eq!(state.history_usage().undo_steps, 0);
    }
    Ok(())
}

#[test]
fn changing_reconnect_policy_in_same_transaction_cannot_authorize_old_edge() -> TestResult {
    let original = BOARD.replace(
        r#""version":1,"fromPort""#,
        r#""version":1,"reconnect":"none","fromPort""#,
    );
    for policy_first in [true, false] {
        let state = session(&original)?;
        let mut commands = vec![
            CanvasCommand::SetReconnectPolicy {
                edge_id: "e".into(),
                policy: ReconnectPolicy::Both,
            },
            reconnect_from(),
        ];
        if !policy_first {
            commands.reverse();
        }
        assert!(
            state
                .prepare(
                    CanvasTransaction::new(state.identity(), commands),
                    &StandardDialect
                )
                .is_err()
        );
        assert_eq!(state.document().raw_json(), original);
    }
    Ok(())
}

#[test]
fn opaque_and_malformed_namespaces_survive_edits_and_refuse_raw_replacement() -> TestResult {
    for opaque in [
        r#"{"version":7,"privateUnknown":1.23000}"#,
        "42",
        r#"{"version":1,"ports":null}"#,
    ] {
        let original = format!(
            r#"{{"nodes":[{{"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"old","craftCanvas":{opaque}}}],"edges":[]}}"#
        );
        let mut state = session(&original)?;
        let prepared = state.prepare(
            CanvasTransaction::new(
                state.identity(),
                vec![CanvasCommand::EditText {
                    node_id: "a".into(),
                    expected: "old".into(),
                    text: "new".into(),
                }],
            ),
            &StandardDialect,
        )?;
        state.apply(prepared)?;
        assert_eq!(
            state
                .document()
                .node("a")
                .ok_or("a")?
                .field("craftCanvas")
                .ok_or("extension")?
                .get(),
            opaque
        );
        for update in [
            FieldUpdate::set(
                vec!["craftCanvas".into()],
                r#"{"version":1,"ports":[]}"#,
                state.limits(),
            )?,
            FieldUpdate::remove(vec!["craftCanvas".into()], state.limits())?,
            FieldUpdate::set(
                vec!["craftCanvas".into(), "version".into()],
                "2",
                state.limits(),
            )?,
        ] {
            assert!(
                state
                    .prepare(
                        CanvasTransaction::new(
                            state.identity(),
                            vec![CanvasCommand::SetNodeFields {
                                node_id: "a".into(),
                                fields: vec![update]
                            }]
                        ),
                        &StandardDialect
                    )
                    .is_err()
            );
        }
        state.apply(state.prepare_undo()?)?;
        assert_eq!(state.document().raw_json(), original);
    }
    Ok(())
}

#[test]
fn duplication_preserves_imported_unresolved_port_without_fabricated_fallback() -> TestResult {
    let original = BOARD.replace(r#""fromPort":{"id":"out""#, r#""fromPort":{"id":"missing""#);
    let mut state = session(&original)?;
    let ids = IdMapping {
        nodes: BTreeMap::from([("a".into(), "a2".into()), ("b".into(), "b2".into())]),
        edges: BTreeMap::from([("e".into(), "e2".into())]),
    };
    let prepared = state.prepare(
        CanvasTransaction::new(
            state.identity(),
            vec![
                CanvasCommand::Duplicate {
                    node_ids: vec!["a".into(), "b".into()],
                    ids,
                    delta: WorldPoint { x: 0.0, y: 200.0 },
                },
                CanvasCommand::Duplicate {
                    node_ids: vec!["a2".into(), "b2".into()],
                    ids: IdMapping {
                        nodes: BTreeMap::from([
                            ("a2".into(), "a3".into()),
                            ("b2".into(), "b3".into()),
                        ]),
                        edges: BTreeMap::from([("e2".into(), "e3".into())]),
                    },
                    delta: WorldPoint { x: 0.0, y: 200.0 },
                },
            ],
        ),
        &StandardDialect,
    )?;
    state.apply(prepared)?;
    assert_eq!(
        state
            .document()
            .edge("e2")
            .ok_or("e2")?
            .endpoint(ConnectorEndpointKind::From)?,
        endpoint("a2", "missing", PortKind::Source)
    );
    assert!(
        state
            .document()
            .node("a2")
            .ok_or("a2")?
            .port(&PortKey {
                id: "missing".into(),
                kind: PortKind::Source
            })
            .is_none()
    );
    assert_eq!(
        state
            .document()
            .edge("e3")
            .ok_or("e3")?
            .endpoint(ConnectorEndpointKind::From)?,
        endpoint("a3", "missing", PortKind::Source)
    );
    state.apply(state.prepare_undo()?)?;
    assert_eq!(state.document().raw_json(), original);
    Ok(())
}

#[test]
fn stale_named_reconnect_cannot_apply_after_port_update() -> TestResult {
    let mut state = session(BOARD)?;
    let old_request = CanvasTransaction::new(state.identity(), vec![reconnect_from()]);
    let old_prepared = state.prepare(
        CanvasTransaction::new(state.identity(), vec![reconnect_from()]),
        &StandardDialect,
    )?;
    let mut ports = state.document().node("a").ok_or("a")?.ports().to_vec();
    ports[1].start = false;
    let update = state.prepare(
        CanvasTransaction::new(
            state.identity(),
            vec![CanvasCommand::SetPorts {
                node_id: "a".into(),
                ports,
            }],
        ),
        &StandardDialect,
    )?;
    state.apply(update)?;
    let updated = state.document().raw_json().to_owned();
    assert!(matches!(
        state.prepare(old_request, &StandardDialect),
        Err(CanvasError::StaleBase)
    ));
    assert!(matches!(
        state.apply(old_prepared),
        Err(CanvasError::StaleBase)
    ));
    assert_eq!(state.document().raw_json(), updated);
    Ok(())
}

fn duplicate_pair() -> CanvasCommand {
    CanvasCommand::Duplicate {
        node_ids: vec!["a".into(), "b".into()],
        ids: IdMapping {
            nodes: BTreeMap::from([("a".into(), "a2".into()), ("b".into(), "b2".into())]),
            edges: BTreeMap::from([("e".into(), "e2".into())]),
        },
        delta: WorldPoint { x: 0.0, y: 200.0 },
    }
}

#[test]
fn retargeting_a_copy_in_the_same_transaction_receives_full_validation() -> TestResult {
    let state = session(BOARD)?;
    let command = CanvasCommand::ReconnectPort {
        edge_id: "e2".into(),
        endpoint: ConnectorEndpointKind::From,
        expected: endpoint("a2", "out", PortKind::Source),
        // This port exists, but its target kind cannot be a strict source.
        replacement: endpoint("b2", "in", PortKind::Target),
    };
    assert!(matches!(
        state.prepare(
            CanvasTransaction::new(state.identity(), vec![duplicate_pair(), command]),
            &StandardDialect
        ),
        Err(CanvasError::Connection(ConnectionError::DirectionMismatch))
    ));
    assert_eq!(state.document().raw_json(), BOARD);
    assert_eq!(state.history_usage().undo_steps, 0);
    Ok(())
}

#[test]
fn deleting_then_reinserting_the_same_edge_id_does_not_bypass_original_policy() -> TestResult {
    let original = BOARD.replace(
        r#""version":1,"fromPort""#,
        r#""version":1,"reconnect":"none","fromPort""#,
    );
    let state = session(&original)?;
    let from = endpoint("a", "aux", PortKind::Source);
    let to = endpoint("b", "in", PortKind::Target);
    let replacement = EdgeRecord::connect_ports("e".into(), from, to, state.limits())?;
    let removal = CanvasCommand::RemoveSelection(Selection {
        edges: ["e".into()].into_iter().collect(),
        ..Selection::default()
    });
    let insertion = CanvasCommand::InsertEdge {
        edge: replacement,
        index: 0,
    };
    assert!(matches!(
        state.prepare(
            CanvasTransaction::new(state.identity(), vec![removal, insertion]),
            &StandardDialect
        ),
        Err(CanvasError::Connection(
            ConnectionError::ReconnectDisabled { .. }
        ))
    ));
    assert_eq!(state.document().raw_json(), original);
    Ok(())
}

struct ExcludeCopies;
impl CanvasDialect for ExcludeCopies {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        StandardDialect.node_policy(node)
    }
    fn can_connect(&self, node: &NodeRecord) -> bool {
        node.id() != "b2"
    }
}
#[test]
fn preserved_copies_still_require_legacy_host_node_eligibility() -> TestResult {
    let state = session(BOARD)?;
    assert!(
        matches!(state.prepare(CanvasTransaction::new(state.identity(), vec![duplicate_pair()]), &ExcludeCopies), Err(CanvasError::NotEditable(id)) if id == "b2")
    );
    assert_eq!(state.document().raw_json(), BOARD);
    Ok(())
}

#[test]
fn raw_port_catalog_rebuild_charges_the_same_shared_work_budget() -> TestResult {
    let ports: Vec<_> = (0..32)
        .map(|i| format!(r#"{{"id":"p{i}","kind":"source","side":"right","x":100,"y":{i}}}"#))
        .collect();
    let original = format!(
        r#"{{"nodes":[{{"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"a","craftCanvas":{{"version":1,"ports":[{}]}}}}],"edges":[]}}"#,
        ports.join(",")
    );
    let state = session(&original)?;
    let update = FieldUpdate::set(
        vec!["craftCanvas".into(), "future".into()],
        "true",
        state.limits(),
    )?;
    let request = CanvasTransaction::new(
        state.identity(),
        vec![CanvasCommand::SetNodeFields {
            node_id: "a".into(),
            fields: vec![update],
        }],
    );
    let mut work = WorkControl::new(50, None);
    assert!(matches!(
        state.prepare_with_control(request, &StandardDialect, &mut work),
        Err(CanvasError::WorkLimit)
    ));
    assert_eq!(state.document().raw_json(), original);
    Ok(())
}
