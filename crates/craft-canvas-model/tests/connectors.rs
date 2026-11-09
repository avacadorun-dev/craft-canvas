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
