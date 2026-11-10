//! Synthetic forward-removal policy and compatible preservation contracts.
use craft_canvas_model::*;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const BOARD: &str = r#"
 {"future":{"precise":123456789012345678901234567890.12300,"huge":1e9999},"nodes":[
 { "id":"a", "type":"text", "x":0,"y":0,"width":100,"height":100,"text":"Привет 日本語","protected":false },
 {"id":"b","type":"text","x":200,"y":0,"width":100,"height":100,"text":"b","future":1.23000},
 {"id":"g","type":"group","x":-40,"y":-40,"width":600,"height":200,"label":"Group"},
 {"id":"u","type":"future-kind","x":400,"y":0,"width":100,"height":100,"protected":true,"locked":true,"hidden":true,"craftCanvas":{"version":9,"future":1.23000}}
 ],"edges":[
 { "id":"e", "fromNode":"a","toNode":"b","fromSide":"right","toSide":"left","protected":false,"future":2.3400 },
 {"id":"loop","fromNode":"a","toNode":"a","fromSide":"right","toSide":"left","protected":false}
 ]}
 "#;

struct ProtectedFields;
impl CanvasDialect for ProtectedFields {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        StandardDialect.node_policy(node)
    }
    fn node_interaction(&self, node: &NodeRecord) -> NodeInteractionPolicy {
        NodeInteractionPolicy {
            deletable: !node
                .field("protected")
                .is_some_and(|raw| raw.get() == "true"),
            ..NodeInteractionPolicy::default()
        }
    }
    fn edge_interaction(&self, edge: &EdgeRecord) -> EdgeInteractionPolicy {
        EdgeInteractionPolicy {
            deletable: !edge
                .field("protected")
                .is_some_and(|raw| raw.get() == "true"),
            ..EdgeInteractionPolicy::default()
        }
    }
}

#[derive(Clone, Copy)]
enum Target {
    Node(&'static str),
    Edge(&'static str),
}
impl Target {
    fn id(self) -> &'static str {
        match self {
            Self::Node(id) | Self::Edge(id) => id,
        }
    }
}

fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(CanvasDocument::parse(BOARD, &limits)?, limits)
}

fn remove(nodes: &[&str], edges: &[&str]) -> CanvasCommand {
    CanvasCommand::RemoveSelection(Selection {
        nodes: nodes.iter().map(|id| (*id).to_owned()).collect(),
        edges: edges.iter().map(|id| (*id).to_owned()).collect(),
    })
}

fn set_protection(target: Target, protected: bool, limits: &CanvasLimits) -> Result<CanvasCommand> {
    let fields = vec![FieldUpdate::set(
        vec!["protected".into()],
        if protected { "true" } else { "false" },
        limits,
    )?];
    Ok(match target {
        Target::Node(id) => CanvasCommand::SetNodeFields {
            node_id: id.into(),
            fields,
        },
        Target::Edge(id) => CanvasCommand::SetEdgeFields {
            edge_id: id.into(),
            fields,
        },
    })
}

fn assert_not_editable(state: &CanvasSession, commands: Vec<CanvasCommand>, protected: &str) {
    let before = state.document().raw_json().to_owned();
    let identity = state.identity();
    let history = state.history_usage();
    let result = state.prepare(CanvasTransaction::new(identity, commands), &ProtectedFields);
    assert!(matches!(result, Err(CanvasError::NotEditable(id)) if id == protected));
    assert_eq!(state.document().raw_json(), before);
    assert_eq!(state.identity(), identity);
    assert_eq!(state.history_usage(), history);
}

#[test]
fn defaults_preserve_unknown_records_group_members_and_exact_history() -> TestResult {
    let mut state = session()?;
    let unknown = state.document().node("u").ok_or("u")?;
    assert_eq!(
        StandardDialect.node_interaction(unknown),
        NodeInteractionPolicy {
            selectable: true,
            deletable: true,
            focusable: true,
            hidden: false,
        }
    );
    assert!(!StandardDialect.node_policy(unknown).movable);
    assert_eq!(
        StandardDialect.edge_interaction(state.document().edge("e").ok_or("e")?),
        EdgeInteractionPolicy {
            selectable: true,
            deletable: true,
            hidden: false,
        }
    );
    let group_change = state.prepare(
        CanvasTransaction::new(state.identity(), vec![remove(&["g"], &[])]),
        &ProtectedFields,
    )?;
    for id in ["a", "b", "u"] {
        assert_eq!(
            group_change.document().node(id).ok_or(id)?.raw_json(),
            state.document().node(id).ok_or(id)?.raw_json()
        );
    }
    for id in ["e", "loop"] {
        assert_eq!(
            group_change.document().edge(id).ok_or(id)?.raw_json(),
            state.document().edge(id).ok_or(id)?.raw_json()
        );
    }
    assert_eq!(
        group_change
            .document()
            .nodes()
            .iter()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        ["a", "b", "u"]
    );
    assert_eq!(
        group_change
            .document()
            .field("future")
            .ok_or("future")?
            .get(),
        state.document().field("future").ok_or("future")?.get()
    );
    assert_eq!(
        group_change.selection_map().removed_nodes,
        ["g".to_owned()].into_iter().collect()
    );
    assert!(group_change.selection_map().removed_edges.is_empty());
    state.apply(group_change)?;
    let without_group = state.document().raw_json().to_owned();
    state.apply(state.prepare_undo()?)?;
    assert_eq!(state.document().raw_json(), BOARD);
    state.apply(state.prepare_redo()?)?;
    assert_eq!(state.document().raw_json(), without_group);

    let unknown_change = state.prepare(
        CanvasTransaction::new(state.identity(), vec![remove(&["u"], &[])]),
        &StandardDialect,
    )?;
    state.apply(unknown_change)?;
    assert!(state.document().node("u").is_none());
    state.apply(state.prepare_undo()?)?;
    assert_eq!(state.document().raw_json(), without_group);
    Ok(())
}

#[test]
fn protected_nodes_explicit_edges_and_incident_edges_refuse_field_edit_bypass() -> TestResult {
    for (target, nodes, edges) in [
        (Target::Node("a"), &["a", "g"][..], &[][..]),
        (Target::Edge("e"), &["g"][..], &["e"][..]),
        (Target::Edge("e"), &["a", "g"][..], &[][..]),
        (Target::Edge("loop"), &["a", "g"][..], &[][..]),
    ] {
        let mut state = session()?;
        let protection = state.prepare(
            CanvasTransaction::new(
                state.identity(),
                vec![set_protection(target, true, state.limits())?],
            ),
            &ProtectedFields,
        )?;
        state.apply(protection)?;
        assert_not_editable(
            &state,
            vec![
                CanvasCommand::EditText {
                    node_id: "b".into(),
                    expected: "b".into(),
                    text: "must remain uncommitted".into(),
                },
                remove(nodes, edges),
            ],
            target.id(),
        );
        assert_not_editable(
            &state,
            vec![
                set_protection(target, false, state.limits())?,
                remove(nodes, edges),
            ],
            target.id(),
        );

        let before = state.document().raw_json().to_owned();
        assert!(matches!(
            state.prepare(
                CanvasTransaction::new(state.identity(), vec![remove(nodes, &["missing"])]),
                &ProtectedFields
            ),
            Err(CanvasError::NotFound(id)) if id == "missing"
        ));
        assert_eq!(state.document().raw_json(), before);

        let unlock = state.prepare(
            CanvasTransaction::new(
                state.identity(),
                vec![set_protection(target, false, state.limits())?],
            ),
            &ProtectedFields,
        )?;
        state.apply(unlock)?;
        state.prepare(
            CanvasTransaction::new(state.identity(), vec![remove(nodes, edges)]),
            &ProtectedFields,
        )?;
    }
    Ok(())
}

#[test]
fn current_candidate_protection_refuses_existing_and_new_record_removal() -> TestResult {
    let state = session()?;
    for (target, removal) in [
        (Target::Node("a"), remove(&["a"], &[])),
        (Target::Edge("e"), remove(&["a"], &[])),
    ] {
        assert_not_editable(
            &state,
            vec![set_protection(target, true, state.limits())?, removal],
            target.id(),
        );
    }
    let node = NodeRecord::parse(
        r#"{"id":"new-node","type":"text","x":0,"y":200,"width":100,"height":100,"text":"new","protected":true}"#,
        state.limits(),
    )?;
    assert_not_editable(
        &state,
        vec![
            CanvasCommand::InsertNode {
                node,
                index: state.document().nodes().len(),
            },
            remove(&["new-node"], &[]),
        ],
        "new-node",
    );
    let edge = EdgeRecord::parse(
        r#"{"id":"new-edge","fromNode":"a","toNode":"b","protected":true}"#,
        state.limits(),
    )?;
    assert_not_editable(
        &state,
        vec![
            CanvasCommand::InsertEdge {
                edge,
                index: state.document().edges().len(),
            },
            remove(&["a"], &[]),
        ],
        "new-edge",
    );
    Ok(())
}

#[test]
fn protected_edge_reconnected_away_survives_removal_with_exact_history() -> TestResult {
    let mut state = session()?;
    let protection = state.prepare(
        CanvasTransaction::new(
            state.identity(),
            vec![set_protection(Target::Edge("e"), true, state.limits())?],
        ),
        &ProtectedFields,
    )?;
    state.apply(protection)?;
    let before = state.document().raw_json().to_owned();
    let change = state.prepare(
        CanvasTransaction::new(
            state.identity(),
            vec![
                CanvasCommand::ReconnectEdge {
                    edge_id: "e".into(),
                    endpoint: ConnectorEndpointKind::From,
                    expected: ConnectorEndpoint {
                        node_id: "a".into(),
                        side: Some(Side::Right),
                    },
                    replacement: ConnectorEndpoint {
                        node_id: "g".into(),
                        side: Some(Side::Right),
                    },
                },
                remove(&["a"], &[]),
            ],
        ),
        &ProtectedFields,
    )?;
    assert_eq!(
        change.selection_map().removed_edges,
        ["loop".to_owned()].into_iter().collect()
    );
    assert_eq!(change.document().edge("e").ok_or("e")?.from_node(), "g");
    assert_eq!(
        change
            .document()
            .edge("e")
            .ok_or("e")?
            .field("future")
            .ok_or("future")?
            .get(),
        "2.3400"
    );
    state.apply(change)?;
    let after = state.document().raw_json().to_owned();
    state.apply(state.prepare_undo()?)?;
    assert_eq!(state.document().raw_json(), before);
    state.apply(state.prepare_redo()?)?;
    assert_eq!(state.document().raw_json(), after);
    Ok(())
}
