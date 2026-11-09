//! Synthetic pure-preview contracts for named ports and reconnect authorization.
use craft_canvas_model::*;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

const RAW: &str = r#"{
  "nodes": [
    {"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"a",
     "craftCanvas":{"version":1,"ports":[
       {"id":"s","kind":"source","side":"right","x":100,"y":25},
       {"id":"t","kind":"target","side":"right","x":100,"y":75},
       {"id":"shared","kind":"source","side":"right","x":100,"y":40},
       {"id":"shared","kind":"target","side":"right","x":100,"y":60}]}},
    {"id":"b","type":"text","x":200,"y":0,"width":100,"height":100,"text":"b",
     "craftCanvas":{"version":1,"ports":[
       {"id":"s","kind":"source","side":"left","x":0,"y":25,"start":false},
       {"id":"t","kind":"target","side":"left","x":0,"y":75,"end":false}]}},
    {"id":"c","type":"text","x":400,"y":0,"width":100,"height":100,"text":"c",
     "craftCanvas":{"version":1,"ports":[
       {"id":"s","kind":"source","side":"left","x":0,"y":25},
       {"id":"t","kind":"target","side":"left","x":0,"y":75}]}}
  ],
  "edges":[{"id":"e","fromNode":"a","toNode":"c","craftCanvas":{"version":1,
    "fromPort":{"id":"s","kind":"source"},"toPort":{"id":"t","kind":"target"},"reconnect":"to"}}]
}"#;

fn document() -> Result<CanvasDocument> {
    CanvasDocument::parse(RAW, &CanvasLimits::default())
}

fn named(node: &str, id: &str, kind: PortKind) -> PortEndpoint {
    PortEndpoint {
        node_id: node.into(),
        binding: PortBinding::Named(PortKey {
            id: id.into(),
            kind,
        }),
    }
}

fn request() -> ConnectionRequest {
    ConnectionRequest {
        from: named("a", "s", PortKind::Source),
        to: named("c", "t", PortKind::Target),
        start: ConnectorEndpointKind::From,
        operation: ConnectionOperation::Create {
            edge_id: "new".into(),
        },
    }
}

struct Loose;
impl CanvasDialect for Loose {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        StandardDialect.node_policy(node)
    }
    fn connection_mode(&self) -> ConnectionMode {
        ConnectionMode::Loose
    }
}

#[test]
fn strict_direction_and_loose_handle_identity_follow_pinned_semantics() -> TestResult {
    let document = document()?;
    let mut request = request();
    request.to = named("b", "s", PortKind::Source);
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::DirectionMismatch)
    );
    validate_connection(&document, &request, &Loose)?;

    request.from = named("a", "shared", PortKind::Source);
    request.to = named("a", "shared", PortKind::Target);
    validate_connection(&document, &request, &StandardDialect)?;
    assert_eq!(
        validate_connection(&document, &request, &Loose),
        Err(ConnectionError::SamePort)
    );
    request.to = named("a", "t", PortKind::Target);
    validate_connection(&document, &request, &Loose)?;
    Ok(())
}

#[test]
fn target_start_checks_actual_start_and_end_availability() -> TestResult {
    let document = document()?;
    let mut request = request();
    request.to = named("b", "t", PortKind::Target);
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::EndUnavailable(request.to.clone()))
    );
    request.start = ConnectorEndpointKind::To;
    validate_connection(&document, &request, &StandardDialect)?;

    request.from = named("b", "s", PortKind::Source);
    request.to = named("c", "t", PortKind::Target);
    validate_connection(&document, &request, &StandardDialect)?;
    request.start = ConnectorEndpointKind::From;
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::StartUnavailable(request.from.clone()))
    );
    Ok(())
}

#[test]
fn reconnect_obeys_original_per_end_policy_and_stationary_binding() -> TestResult {
    let document = document()?;
    let mut request = request();
    request.operation = ConnectionOperation::Reconnect {
        edge_id: "e".into(),
        endpoint: ConnectorEndpointKind::To,
    };
    request.to = named("a", "t", PortKind::Target);
    validate_connection(&document, &request, &StandardDialect)?;
    request.from = named("c", "s", PortKind::Source);
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::StationaryEndpointChanged("e".into()))
    );
    request.operation = ConnectionOperation::ReconnectBoth {
        edge_id: "e".into(),
    };
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::ReconnectDisabled {
            edge_id: "e".into(),
            endpoint: ConnectorEndpointKind::From,
        })
    );
    Ok(())
}

#[test]
fn reconnect_preview_refuses_an_initiator_that_would_bypass_final_availability() -> TestResult {
    let raw = RAW
        .replace(r#""reconnect":"to""#, r#""reconnect":"both""#)
        .replace(
            r#""id":"shared","kind":"source","side":"right","x":100,"y":40"#,
            r#""id":"shared","kind":"source","side":"right","x":100,"y":40,"end":false"#,
        );
    let document = CanvasDocument::parse(&raw, &CanvasLimits::default())?;
    let mut request = request();
    request.from = named("a", "shared", PortKind::Source);
    request.operation = ConnectionOperation::Reconnect {
        edge_id: "e".into(),
        endpoint: ConnectorEndpointKind::From,
    };
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::InvalidInitiator {
            expected: ConnectorEndpointKind::To,
            actual: ConnectorEndpointKind::From,
        })
    );
    request.start = ConnectorEndpointKind::To;
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::EndUnavailable(request.from.clone()))
    );

    request.operation = ConnectionOperation::ReconnectBoth {
        edge_id: "e".into(),
    };
    request.to = named("a", "t", PortKind::Target);
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::InvalidInitiator {
            expected: ConnectorEndpointKind::From,
            actual: ConnectorEndpointKind::To,
        })
    );
    request.start = ConnectorEndpointKind::From;
    validate_connection(&document, &request, &StandardDialect)?;
    Ok(())
}

#[test]
fn named_bindings_refuse_opaque_ports_but_legacy_bindings_remain_available() -> TestResult {
    let raw = RAW.replace(r#""version":1,"ports""#, r#""version":8,"ports""#);
    let document = CanvasDocument::parse(&raw, &CanvasLimits::default())?;
    let mut request = request();
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::OpaqueExtension("a".into()))
    );
    request.from.binding = PortBinding::Legacy(None);
    request.to.binding = PortBinding::Legacy(Some(Side::Left));
    validate_connection(&document, &request, &StandardDialect)?;
    assert_eq!(document.raw_json(), raw);
    Ok(())
}

struct RejectParallel;
impl CanvasDialect for RejectParallel {
    fn node_policy(&self, node: &NodeRecord) -> NodePolicy {
        StandardDialect.node_policy(node)
    }
    fn validate_connection(
        &self,
        document: &CanvasDocument,
        request: &ConnectionRequest,
    ) -> std::result::Result<(), ConnectionError> {
        if let ConnectionOperation::Create { edge_id } = &request.operation
            && document.edges().iter().any(|edge| {
                edge.id() != edge_id
                    && edge.from_node() == request.from.node_id
                    && edge.to_node() == request.to.node_id
            })
        {
            return Err(ConnectionError::HostRejected(
                "The selected host excludes parallel edges".into(),
            ));
        }
        Ok(())
    }
}

#[test]
fn parallel_edges_are_allowed_unless_the_host_refuses_them() -> TestResult {
    let document = document()?;
    let mut request = request();
    validate_connection(&document, &request, &StandardDialect)?;
    assert!(matches!(
        validate_connection(&document, &request, &RejectParallel),
        Err(ConnectionError::HostRejected(_))
    ));
    // The final candidate includes the new edge; its planned ID lets the same
    // host check avoid counting that edge as its own parallel competitor.
    request.operation = ConnectionOperation::Create {
        edge_id: "e".into(),
    };
    validate_connection(&document, &request, &RejectParallel)?;
    assert_eq!(document.raw_json(), RAW);
    Ok(())
}

#[test]
fn missing_named_port_never_falls_back_to_another_port_on_its_side() -> TestResult {
    let document = document()?;
    let mut request = request();
    request.to = named("c", "missing", PortKind::Target);
    assert_eq!(
        validate_connection(&document, &request, &StandardDialect),
        Err(ConnectionError::MissingPort {
            node_id: "c".into(),
            port: PortKey {
                id: "missing".into(),
                kind: PortKind::Target,
            },
        })
    );
    Ok(())
}
