//! Synthetic connector command and derived-geometry regression contracts.
use craft_canvas_model::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
const RAW: &str = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"a"},{"id":"b","type":"text","x":200,"y":0,"width":100,"height":100,"text":"b"},{"id":"c","type":"text","x":200,"y":200,"width":100,"height":100,"text":"c"}],"edges":[{"id":"e","fromNode":"a","fromSide":"right","fromEnd":"\u0061rrow","toNode":"b","toSide":"left","label":"old","future":{"n":1.23000}}]}"#;
fn session() -> Result<CanvasSession> {
    let limits = CanvasLimits::default();
    CanvasSession::new(CanvasDocument::parse(RAW, &limits)?, limits)
}
#[test]
fn decoded_markers_and_missing_target_default_are_preserved() -> TestResult {
    let session = session()?;
    let edge = session.document().edge("e").ok_or("e")?;
    assert_eq!(edge.from_end(), ConnectorEnd::Arrow);
    assert_eq!(edge.to_end(), ConnectorEnd::Arrow);
    assert!(edge.raw_json().contains("\\u0061rrow"));
    Ok(())
}
#[test]
fn reconnect_and_label_keep_identity_unknown_numbers_and_one_exact_undo() -> TestResult {
    let mut session = session()?;
    let commands = vec![
        CanvasCommand::ReconnectEdge {
            edge_id: "e".into(),
            endpoint: ConnectorEndpointKind::To,
            expected: ConnectorEndpoint {
                node_id: "b".into(),
                side: Some(Side::Left),
            },
            replacement: ConnectorEndpoint {
                node_id: "c".into(),
                side: Some(Side::Top),
            },
        },
        CanvasCommand::EditLabel {
            target: LabelTarget::Edge("e".into()),
            expected: "old".into(),
            label: "\u{feff}  Новая связь 🌙  ".into(),
        },
    ];
    let change = session.prepare(
        CanvasTransaction::new(session.identity(), commands),
        &StandardDialect,
    )?;
    assert_eq!(session.document().raw_json(), RAW);
    session.apply(change)?;
    let edge = session.document().edge("e").ok_or("e")?;
    assert_eq!(edge.id(), "e");
    assert_eq!(edge.to_node(), "c");
    assert_eq!(edge.to_side(), Some(Side::Top));
    assert_eq!(edge.label(), "Новая связь 🌙");
    assert_eq!(
        edge.field("future").ok_or("future")?.get(),
        r#"{"n":1.23000}"#
    );
    assert_eq!(session.history_usage().undo_steps, 1);
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), RAW);
    Ok(())
}
#[test]
fn wrong_expected_endpoint_refuses_without_mutation() -> TestResult {
    let session = session()?;
    let command = CanvasCommand::ReconnectEdge {
        edge_id: "e".into(),
        endpoint: ConnectorEndpointKind::To,
        expected: ConnectorEndpoint {
            node_id: "c".into(),
            side: Some(Side::Left),
        },
        replacement: ConnectorEndpoint {
            node_id: "a".into(),
            side: Some(Side::Top),
        },
    };
    assert!(matches!(
        session.prepare(
            CanvasTransaction::new(session.identity(), vec![command]),
            &StandardDialect
        ),
        Err(CanvasError::StaleBase)
    ));
    assert_eq!(session.document().raw_json(), RAW);
    Ok(())
}

#[test]
fn node_only_reconnect_preserves_unchanged_legacy_side_lexeme() -> TestResult {
    for side_json in ["null", r#""""#, r#""\u006ceft""#] {
        let original = RAW.replace(r#""toSide":"left""#, &format!(r#""toSide":{side_json}"#));
        let limits = CanvasLimits::default();
        let mut session = CanvasSession::new(CanvasDocument::parse(&original, &limits)?, limits)?;
        let side = session.document().edge("e").ok_or("e")?.to_side();
        let prepared = session.prepare(
            CanvasTransaction::new(
                session.identity(),
                vec![CanvasCommand::ReconnectEdge {
                    edge_id: "e".into(),
                    endpoint: ConnectorEndpointKind::To,
                    expected: ConnectorEndpoint {
                        node_id: "b".into(),
                        side,
                    },
                    replacement: ConnectorEndpoint {
                        node_id: "c".into(),
                        side,
                    },
                }],
            ),
            &StandardDialect,
        )?;
        session.apply(prepared)?;
        assert_eq!(
            session
                .document()
                .edge("e")
                .ok_or("e")?
                .field("toSide")
                .ok_or("side")?
                .get(),
            side_json
        );
        session.apply(session.prepare_undo()?)?;
        assert_eq!(session.document().raw_json(), original);
    }
    Ok(())
}

struct RefuseNodeConnections(&'static str);
impl CanvasDialect for RefuseNodeConnections {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        StandardDialect.node_policy(node)
    }
    fn can_connect(&self, node: &NodeRecord) -> bool {
        node.id() != self.0
    }
}

#[test]
fn field_endpoint_edit_cannot_bypass_final_connection_policy() -> TestResult {
    let session = session()?;
    let request = CanvasTransaction::new(
        session.identity(),
        vec![CanvasCommand::SetEdgeFields {
            edge_id: "e".into(),
            fields: vec![FieldUpdate::set(
                vec!["toNode".into()],
                r#""c""#,
                session.limits(),
            )?],
        }],
    );
    assert!(matches!(
        session.prepare(request, &RefuseNodeConnections("c")),
        Err(CanvasError::NotEditable(id)) if id == "c"
    ));
    assert_eq!(session.document().raw_json(), RAW);
    assert_eq!(session.history_usage().undo_steps, 0);
    Ok(())
}

#[test]
fn unrelated_field_edit_preserves_an_existing_nonconnectable_edge() -> TestResult {
    let mut session = session()?;
    let request = CanvasTransaction::new(
        session.identity(),
        vec![CanvasCommand::SetEdgeFields {
            edge_id: "e".into(),
            fields: vec![FieldUpdate::set(
                vec!["toEnd".into()],
                r#""none""#,
                session.limits(),
            )?],
        }],
    );
    let prepared = session.prepare(request, &RefuseNodeConnections("b"))?;
    session.apply(prepared)?;
    let edge = session.document().edge("e").ok_or("edge")?;
    assert_eq!(edge.to_node(), "b");
    assert_eq!(edge.to_end(), ConnectorEnd::None);
    assert_eq!(
        edge.field("future").ok_or("future")?.get(),
        r#"{"n":1.23000}"#
    );
    session.apply(session.prepare_undo()?)?;
    assert_eq!(session.document().raw_json(), RAW);
    Ok(())
}

#[test]
fn bezier_controls_match_source_and_flatten_is_bounded() -> TestResult {
    let curve = CubicConnector::between(
        WorldPoint { x: 0., y: 0. },
        Side::Right,
        WorldPoint { x: 200., y: 100. },
        Side::Left,
    )?;
    assert_eq!(curve.control_from, WorldPoint { x: 100., y: 0. });
    assert_eq!(curve.control_to, WorldPoint { x: 100., y: 100. });
    assert_eq!(curve.at(0.5)?, WorldPoint { x: 100., y: 50. });
    let mut work = WorkControl::new(10000, None);
    let points = curve.flatten(0.5, 256, &mut work)?;
    assert_eq!(points.first(), Some(&curve.from));
    assert_eq!(points.last(), Some(&curve.to));
    assert!(points.len() > 2);
    let mut work = WorkControl::new(10000, None);
    assert!(curve.flatten(0.00001, 1, &mut work).is_err());
    assert!(
        CubicConnector::between(
            WorldPoint { x: f64::MAX, y: 0. },
            Side::Left,
            WorldPoint {
                x: -f64::MAX,
                y: 0.
            },
            Side::Right
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn smooth_step_matches_source_waypoints_and_label_instead_of_a_straight_or_bezier_line()
-> TestResult {
    let route = SmoothStepConnector::between(
        WorldPoint { x: 0.0, y: 0.0 },
        Side::Right,
        WorldPoint { x: 200.0, y: 100.0 },
        Side::Left,
    )?;
    assert_eq!(
        route.waypoints(),
        &[
            WorldPoint { x: 0.0, y: 0.0 },
            WorldPoint { x: 20.0, y: 0.0 },
            WorldPoint { x: 100.0, y: 0.0 },
            WorldPoint { x: 100.0, y: 100.0 },
            WorldPoint { x: 180.0, y: 100.0 },
            WorldPoint { x: 200.0, y: 100.0 },
        ]
    );
    assert_eq!(route.label_anchor(), WorldPoint { x: 100.0, y: 50.0 });
    let mut work = WorkControl::new(1000, None);
    let flattened = route.flatten(0.1, 64, &mut work)?;
    assert!(flattened.contains(&WorldPoint { x: 95.0, y: 0.0 }));
    assert!(flattened.contains(&WorldPoint { x: 100.0, y: 5.0 }));
    assert!(!flattened.contains(&WorldPoint { x: 100.0, y: 0.0 }));
    assert!(route.flatten(0.1, 1, &mut work).is_err());
    Ok(())
}

#[test]
fn opposite_side_self_loop_stays_outside_the_card_at_large_world_coordinates() -> TestResult {
    let bounds = WorldRect {
        x: 10_000_000.0,
        y: -10_000_000.0,
        width: 100.0,
        height: 100.0,
    };
    let route = SmoothStepConnector::loop_around(bounds, Side::Top, Side::Bottom)?;
    let mut work = WorkControl::new(1000, None);
    let points = route.flatten(0.1, 64, &mut work)?;
    for segment in points.windows(2) {
        let midpoint = WorldPoint {
            x: segment[0].x + (segment[1].x - segment[0].x) * 0.5,
            y: segment[0].y + (segment[1].y - segment[0].y) * 0.5,
        };
        assert!(
            !bounds.contains(midpoint),
            "loop segment must remain outside the card body"
        );
    }
    assert!(!bounds.contains(route.label_anchor()));
    Ok(())
}

#[test]
fn named_same_side_loop_uses_distinct_exact_anchors_and_legacy_wrapper_is_unchanged() -> TestResult
{
    let bounds = WorldRect {
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 120.0,
    };
    let from = WorldPoint { x: 210.0, y: 45.5 };
    let to = WorldPoint {
        x: 210.0,
        y: 110.25,
    };
    let route = SmoothStepConnector::loop_between(bounds, from, Side::Right, to, Side::Right)?;
    assert_eq!(
        route.waypoints(),
        &[
            from,
            WorldPoint { x: 230.0, y: 45.5 },
            WorldPoint {
                x: 230.0,
                y: 110.25
            },
            to,
        ]
    );
    let mut work = WorkControl::new(1000, None);
    let path = route.flatten(0.1, 64, &mut work)?;
    assert_eq!(path.first(), Some(&from));
    assert_eq!(path.last(), Some(&to));
    assert!(!bounds.contains(route.label_anchor()));
    let legacy = SmoothStepConnector::loop_around(bounds, Side::Top, Side::Bottom)?;
    assert_eq!(
        legacy.waypoints(),
        &[
            WorldPoint { x: 110.0, y: 20.0 },
            WorldPoint { x: 110.0, y: 0.0 },
            WorldPoint { x: 230.0, y: 0.0 },
            WorldPoint { x: 230.0, y: 160.0 },
            WorldPoint { x: 110.0, y: 160.0 },
            WorldPoint { x: 110.0, y: 140.0 },
        ]
    );
    let coincident = SmoothStepConnector::loop_around(bounds, Side::Top, Side::Top)?;
    assert_eq!(coincident.waypoints().len(), 8);
    Ok(())
}

#[test]
fn detached_named_loop_expands_its_ring_without_substituting_midpoints() -> TestResult {
    let bounds = WorldRect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let from = WorldPoint {
        x: 180.5,
        y: -30.25,
    };
    let to = WorldPoint {
        x: -25.25,
        y: 130.5,
    };
    let route = SmoothStepConnector::loop_between(bounds, from, Side::Right, to, Side::Left)?;
    assert_eq!(route.waypoints().first(), Some(&from));
    assert_eq!(route.waypoints().last(), Some(&to));
    assert_eq!(
        route.waypoints()[1],
        WorldPoint {
            x: 200.5,
            y: -30.25
        }
    );
    assert!(
        route
            .waypoints()
            .windows(2)
            .all(|pair| pair[0].x == pair[1].x || pair[0].y == pair[1].y)
    );
    assert!(
        SmoothStepConnector::loop_between(
            bounds,
            WorldPoint {
                x: f64::NAN,
                y: 0.0
            },
            Side::Top,
            to,
            Side::Left
        )
        .is_err()
    );
    Ok(())
}
