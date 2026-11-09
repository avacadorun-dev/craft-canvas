//! Synthetic versioned-port codec, preservation and capacity contracts.
use craft_canvas_model::{
    CanvasCommand, CanvasDocument, CanvasError, CanvasLimits, CanvasSession, CanvasTransaction,
    ConnectorEndpointKind, EdgeRecord, ExtensionStatus, NodeRecord, PortBinding, PortEndpoint,
    PortKey, PortKind, ReconnectPolicy, StandardDialect, WorldPoint,
};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn node(extension: &str, limits: &CanvasLimits) -> craft_canvas_model::Result<NodeRecord> {
    NodeRecord::parse(
        &format!(
            r#"{{"id":"n","type":"text","x":10,"y":20,"width":200,"height":120,"text":"Synthetic",{extension}}}"#
        ),
        limits,
    )
}

#[test]
fn same_side_ports_have_distinct_kind_and_id_indexes_with_fractional_local_geometry() -> TestResult
{
    let node = node(
        r#""craftCanvas":{"version":1,"ports":[{"id":"shared","kind":"source","side":"right","x":200.25,"y":30},{"id":"shared","kind":"target","side":"right","x":200.25,"y":90,"start":false}]}"#,
        &CanvasLimits::default(),
    )?;
    let source = node
        .port(&PortKey {
            id: "shared".into(),
            kind: PortKind::Source,
        })
        .ok_or("source")?;
    let target = node
        .port(&PortKey {
            id: "shared".into(),
            kind: PortKind::Target,
        })
        .ok_or("target")?;
    assert_eq!(node.port_extension_status(), ExtensionStatus::KnownV1);
    assert_eq!(
        source.world_anchor(node.bounds())?,
        WorldPoint { x: 210.25, y: 50.0 }
    );
    assert!(source.connectable && source.start && source.end);
    assert!(!target.start);
    assert!(target.connectable && target.end);
    Ok(())
}

#[test]
fn interpretation_distinguishes_absent_opaque_and_malformed_without_rewriting() -> TestResult {
    let limits = CanvasLimits::default();
    for (extension, status) in [
        (r#""other":900719925474099312345"#, ExtensionStatus::Absent),
        (
            r#""craftCanvas":{"version":2,"ports":["future"],"large":900719925474099312345}"#,
            ExtensionStatus::Opaque,
        ),
        (r#""craftCanvas":null"#, ExtensionStatus::Opaque),
        (
            r#""craftCanvas":{"version":1,"ports":[{"id":"broken"}]}"#,
            ExtensionStatus::Malformed,
        ),
        (
            r#""craftCanvas":{"version":1,"ports":null}"#,
            ExtensionStatus::Malformed,
        ),
    ] {
        let parsed = node(extension, &limits)?;
        assert_eq!(parsed.port_extension_status(), status);
        assert!(parsed.ports().is_empty());
        assert!(parsed.raw_json().contains(extension));
    }
    Ok(())
}

#[test]
fn typed_port_update_preserves_unknown_members_default_omissions_and_numeric_lexemes() -> TestResult
{
    let limits = CanvasLimits::default();
    let original = node(
        r#""craftCanvas":{"version":1,"future":{"large":900719925474099312345},"ports":[{"id":"p","kind":"source","side":"right","x":2e2,"y":3e1,"custom":{"precise":1.234567890123456789}}]}"#,
        &limits,
    )?;
    let board = format!(
        "{{\n  \"nodes\":[{}], \"edges\":[]\n}}",
        original.raw_json()
    );
    let mut session = CanvasSession::new(CanvasDocument::parse(&board, &limits)?, limits)?;
    let mut ports = session.document().node("n").ok_or("node")?.ports().to_vec();
    ports[0].end = false;
    let change = session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::SetPorts {
                node_id: "n".into(),
                ports,
            }],
        ),
        &StandardDialect,
    )?;
    let candidate = change
        .document()
        .node("n")
        .ok_or("updated node")?
        .raw_json();
    assert!(candidate.contains(r#""x":2e2"#));
    assert!(candidate.contains(r#""y":3e1"#));
    assert!(candidate.contains(r#""future":{"large":900719925474099312345}"#));
    assert!(candidate.contains(r#""custom":{"precise":1.234567890123456789}"#));
    assert!(!candidate.contains("connectable"));
    assert!(!candidate.contains(r#""start""#));
    assert!(candidate.contains(r#""end":false"#));
    session.apply(change)?;
    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), board);
    Ok(())
}

#[test]
fn semantic_port_noop_retains_complete_original_bytes() -> TestResult {
    let limits = CanvasLimits::default();
    let original = node(
        r#""craftCanvas": {"version":1,"ports":[{"id":"p","kind":"source","side":"top","x":-0.0,"y":0,"start":true}]}"#,
        &limits,
    )?;
    let board = format!("{{ \"nodes\": [ {} ],\"edges\":[] }}", original.raw_json());
    let session = CanvasSession::new(CanvasDocument::parse(&board, &limits)?, limits)?;
    let change = session.prepare(
        CanvasTransaction::new(
            session.identity(),
            vec![CanvasCommand::SetPorts {
                node_id: "n".into(),
                ports: original.ports().to_vec(),
            }],
        ),
        &StandardDialect,
    )?;
    assert!(!change.changed());
    assert_eq!(change.document().raw_json(), board);
    Ok(())
}

#[test]
fn present_malformed_binding_does_not_fall_back_to_legacy_side() -> TestResult {
    let limits = CanvasLimits::default();
    let malformed = EdgeRecord::parse(
        r#"{"id":"e","fromNode":"a","toNode":"b","fromSide":"right","craftCanvas":{"version":1,"fromPort":null}}"#,
        &limits,
    )?;
    assert_eq!(
        malformed.port_extension_status(),
        ExtensionStatus::Malformed
    );
    assert!(malformed.endpoint(ConnectorEndpointKind::From).is_err());
    let legacy = EdgeRecord::parse(
        r#"{"id":"e","fromNode":"a","toNode":"b","fromSide":null}"#,
        &limits,
    )?;
    assert_eq!(
        legacy.endpoint(ConnectorEndpointKind::From)?.binding,
        PortBinding::Legacy(None)
    );
    assert_eq!(legacy.reconnect_policy()?, ReconnectPolicy::Inherit);
    Ok(())
}

#[test]
fn named_edge_constructor_keeps_mixed_legacy_omission_and_named_identity() -> TestResult {
    let from = PortEndpoint {
        node_id: "a".into(),
        binding: PortBinding::Named(PortKey {
            id: "out".into(),
            kind: PortKind::Source,
        }),
    };
    let to = PortEndpoint {
        node_id: "b".into(),
        binding: PortBinding::Legacy(None),
    };
    let edge = EdgeRecord::connect_ports(
        "e".into(),
        from.clone(),
        to.clone(),
        &CanvasLimits::default(),
    )?;
    assert_eq!(edge.endpoint(ConnectorEndpointKind::From)?, from);
    assert_eq!(edge.endpoint(ConnectorEndpointKind::To)?, to);
    assert!(!edge.raw_json().contains("Side"));
    assert!(!edge.raw_json().contains("toPort"));
    Ok(())
}

#[test]
fn recognized_count_limit_refuses_even_when_first_port_is_malformed() -> TestResult {
    let limits = CanvasLimits {
        max_ports_per_node: 1,
        ..CanvasLimits::default()
    };
    let result = node(r#""craftCanvas":{"version":1,"ports":[{},{}]}"#, &limits);
    assert!(matches!(result, Err(CanvasError::Limit("ports per node"))));
    Ok(())
}

#[test]
fn recognized_geometry_and_actual_id_bytes_obey_limits() -> TestResult {
    let limits = CanvasLimits {
        max_coordinate: 1_000.0,
        max_id_bytes: 4,
        ..CanvasLimits::default()
    };
    let coordinate = node(
        r#""craftCanvas":{"version":1,"ports":[{"id":"p","kind":"source","side":"left","x":1001,"y":0}]}"#,
        &limits,
    );
    assert!(matches!(
        coordinate,
        Err(CanvasError::Limit("port anchor coordinate"))
    ));
    let overflow = node(
        r#""craftCanvas":{"version":1,"ports":[{"id":"p","kind":"source","side":"left","x":1e400,"y":0}]}"#,
        &limits,
    );
    assert!(matches!(
        overflow,
        Err(CanvasError::Limit("port anchor coordinate"))
    ));
    let id = node(
        r#""craftCanvas":{"version":1,"ports":[{"id":" p   ","kind":"source","side":"left","x":0,"y":0}]}"#,
        &limits,
    );
    assert!(matches!(id, Err(CanvasError::Limit("port ID bytes"))));
    Ok(())
}
