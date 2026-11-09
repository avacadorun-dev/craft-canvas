use craft_canvas_model::{
    CanvasCommand, CanvasDocument, CanvasError, CanvasLimits, CanvasSession, CanvasTransaction,
    ConnectorEndpointKind, EdgeRecord, ExtensionStatus, NodeRecord, PortBinding, PortEndpoint,
    PortKey, PortKind, ReconnectPolicy, StandardDialect, WorldPoint,
};

fn node(extension: &str, limits: &CanvasLimits) -> craft_canvas_model::Result<NodeRecord> {
    NodeRecord::parse(
        &format!(
            r#"{{"id":"n","type":"text","x":10,"y":20,"width":200,"height":120,"text":"Synthetic",{extension}}}"#
        ),
        limits,
    )
}

#[test]
fn same_side_ports_have_distinct_kind_and_id_indexes_with_fractional_local_geometry() {
    let node = node(r#""craftCanvas":{"version":1,"ports":[{"id":"shared","kind":"source","side":"right","x":200.25,"y":30},{"id":"shared","kind":"target","side":"right","x":200.25,"y":90,"start":false}]}"#, &CanvasLimits::default()).expect("ports");
    let source = node
        .port(&PortKey {
            id: "shared".into(),
            kind: PortKind::Source,
        })
        .expect("source");
    let target = node
        .port(&PortKey {
            id: "shared".into(),
            kind: PortKind::Target,
        })
        .expect("target");
    assert_eq!(node.port_extension_status(), ExtensionStatus::KnownV1);
    assert_eq!(
        source.world_anchor(node.bounds()).expect("anchor"),
        WorldPoint { x: 210.25, y: 50.0 }
    );
    assert!(source.connectable && source.start && source.end);
    assert!(!target.start);
    assert!(target.connectable && target.end);
}

#[test]
fn interpretation_distinguishes_absent_opaque_and_malformed_without_rewriting() {
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
        let parsed = node(extension, &limits).expect("preserved import");
        assert_eq!(parsed.port_extension_status(), status);
        assert!(parsed.ports().is_empty());
        assert!(parsed.raw_json().contains(extension));
    }
}

#[test]
fn typed_port_update_preserves_unknown_members_default_omissions_and_numeric_lexemes() {
    let limits = CanvasLimits::default();
    let original = node(r#""craftCanvas":{"version":1,"future":{"large":900719925474099312345},"ports":[{"id":"p","kind":"source","side":"right","x":2e2,"y":3e1,"custom":{"precise":1.234567890123456789}}]}"#, &limits).expect("node");
    let board = format!(
        "{{\n  \"nodes\":[{}], \"edges\":[]\n}}",
        original.raw_json()
    );
    let mut session = CanvasSession::new(
        CanvasDocument::parse(&board, &limits).expect("board"),
        limits,
    )
    .expect("session");
    let mut ports = session.document().node("n").expect("node").ports().to_vec();
    ports[0].end = false;
    let change = session
        .prepare(
            CanvasTransaction::new(
                session.identity(),
                vec![CanvasCommand::SetPorts {
                    node_id: "n".into(),
                    ports,
                }],
            ),
            &StandardDialect,
        )
        .expect("port update");
    let candidate = change
        .document()
        .node("n")
        .expect("updated node")
        .raw_json();
    assert!(candidate.contains(r#""x":2e2"#));
    assert!(candidate.contains(r#""y":3e1"#));
    assert!(candidate.contains(r#""future":{"large":900719925474099312345}"#));
    assert!(candidate.contains(r#""custom":{"precise":1.234567890123456789}"#));
    assert!(!candidate.contains("connectable"));
    assert!(!candidate.contains(r#""start""#));
    assert!(candidate.contains(r#""end":false"#));
    session.apply(change).expect("apply");
    let undo = session.prepare_undo().expect("undo");
    session.apply(undo).expect("apply undo");
    assert_eq!(session.document().raw_json(), board);
}

#[test]
fn semantic_port_noop_retains_complete_original_bytes() {
    let limits = CanvasLimits::default();
    let original = node(r#""craftCanvas": {"version":1,"ports":[{"id":"p","kind":"source","side":"top","x":-0.0,"y":0,"start":true}]}"#, &limits).expect("node");
    let board = format!("{{ \"nodes\": [ {} ],\"edges\":[] }}", original.raw_json());
    let session = CanvasSession::new(
        CanvasDocument::parse(&board, &limits).expect("board"),
        limits,
    )
    .expect("session");
    let change = session
        .prepare(
            CanvasTransaction::new(
                session.identity(),
                vec![CanvasCommand::SetPorts {
                    node_id: "n".into(),
                    ports: original.ports().to_vec(),
                }],
            ),
            &StandardDialect,
        )
        .expect("noop");
    assert!(!change.changed());
    assert_eq!(change.document().raw_json(), board);
}

#[test]
fn present_malformed_binding_does_not_fall_back_to_legacy_side() {
    let limits = CanvasLimits::default();
    let malformed = EdgeRecord::parse(r#"{"id":"e","fromNode":"a","toNode":"b","fromSide":"right","craftCanvas":{"version":1,"fromPort":null}}"#, &limits).expect("preserved edge");
    assert_eq!(
        malformed.port_extension_status(),
        ExtensionStatus::Malformed
    );
    assert!(malformed.endpoint(ConnectorEndpointKind::From).is_err());
    let legacy = EdgeRecord::parse(
        r#"{"id":"e","fromNode":"a","toNode":"b","fromSide":null}"#,
        &limits,
    )
    .expect("legacy edge");
    assert_eq!(
        legacy
            .endpoint(ConnectorEndpointKind::From)
            .expect("legacy endpoint")
            .binding,
        PortBinding::Legacy(None)
    );
    assert_eq!(
        legacy.reconnect_policy().expect("policy"),
        ReconnectPolicy::Inherit
    );
}

#[test]
fn named_edge_constructor_keeps_mixed_legacy_omission_and_named_identity() {
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
    )
    .expect("edge");
    assert_eq!(
        edge.endpoint(ConnectorEndpointKind::From).expect("from"),
        from
    );
    assert_eq!(edge.endpoint(ConnectorEndpointKind::To).expect("to"), to);
    assert!(!edge.raw_json().contains("Side"));
    assert!(!edge.raw_json().contains("toPort"));
}

#[test]
fn recognized_count_limit_refuses_even_when_first_port_is_malformed() {
    let limits = CanvasLimits {
        max_ports_per_node: 1,
        ..CanvasLimits::default()
    };
    let result = node(r#""craftCanvas":{"version":1,"ports":[{},{}]}"#, &limits);
    assert!(matches!(result, Err(CanvasError::Limit("ports per node"))));
}

#[test]
fn recognized_geometry_and_actual_id_bytes_obey_limits() {
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
}
