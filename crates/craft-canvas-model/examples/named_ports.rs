//! Headless named-port authoring through the same guarded commands as native hosts.
use craft_canvas_model::{
    CanvasCommand, CanvasDocument, CanvasError, CanvasLimits, CanvasSession, CanvasTransaction,
    ConnectionOperation, ConnectionRequest, ConnectorEndpointKind, EdgeRecord, PortBinding,
    PortDefinition, PortEndpoint, PortKey, PortKind, Result, Side, StandardDialect,
    validate_connection,
};

fn key(id: &str, kind: PortKind) -> PortKey {
    PortKey {
        id: id.into(),
        kind,
    }
}

fn port(id: &str, kind: PortKind, y: f64) -> PortDefinition {
    let source = kind == PortKind::Source;
    PortDefinition {
        key: key(id, kind),
        side: if source { Side::Right } else { Side::Left },
        x: if source { 200.0 } else { 0.0 },
        y,
        connectable: true,
        start: true,
        end: true,
    }
}

fn main() -> Result<()> {
    let source = r#"{"nodes":[
        {"id":"a","type":"text","x":0,"y":0,"width":200,"height":120,"text":"Note"},
        {"id":"b","type":"text","x":300,"y":0,"width":200,"height":120,"text":"Scene"}
    ],"edges":[]}"#;
    let limits = CanvasLimits::default();
    let mut session = CanvasSession::new(CanvasDocument::parse(source, &limits)?, limits)?;
    let setup = CanvasTransaction::new(
        session.identity(),
        vec![
            CanvasCommand::SetPorts {
                node_id: "a".into(),
                ports: vec![
                    port("main", PortKind::Source, 30.0),
                    port("alternative", PortKind::Source, 90.0),
                ],
            },
            CanvasCommand::SetPorts {
                node_id: "b".into(),
                ports: vec![port("input", PortKind::Target, 60.0)],
            },
        ],
    );
    session.apply(session.prepare(setup, &StandardDialect)?)?;
    let before_connection = session.document().raw_json().to_owned();
    let edge_id = "note-scene".to_owned();
    let preview = ConnectionRequest {
        from: PortEndpoint {
            node_id: "a".into(),
            binding: PortBinding::Named(key("alternative", PortKind::Source)),
        },
        to: PortEndpoint {
            node_id: "b".into(),
            binding: PortBinding::Named(key("input", PortKind::Target)),
        },
        start: ConnectorEndpointKind::From,
        operation: ConnectionOperation::Create {
            edge_id: edge_id.clone(),
        },
    };
    validate_connection(session.document(), &preview, &StandardDialect)?;
    let edge = EdgeRecord::connect_ports(
        edge_id.clone(),
        preview.from.clone(),
        preview.to.clone(),
        session.limits(),
    )?;
    let transaction = CanvasTransaction::new(
        session.identity(),
        vec![CanvasCommand::ConnectPorts {
            edge,
            index: 0,
            start: preview.start,
        }],
    );
    // Preview grants no authority: prepare checks the current complete candidate again.
    // A persistent host checkpoints this candidate before acknowledging it with apply.
    session.apply(session.prepare(transaction, &StandardDialect)?)?;

    let saved = session.document().raw_json();
    let restored = CanvasDocument::parse(saved, session.limits())?;
    validate_connection(&restored, &preview, &StandardDialect)?;
    let restored_edge = restored
        .edge(&edge_id)
        .ok_or(CanvasError::NotFound(edge_id))?;
    if restored_edge.endpoint(ConnectorEndpointKind::From)? != preview.from
        || restored_edge.endpoint(ConnectorEndpointKind::To)? != preview.to
    {
        return Err(CanvasError::Invalid(
            "named endpoints did not round-trip".into(),
        ));
    }
    session.apply(session.prepare_undo()?)?;
    if session.document().raw_json() != before_connection {
        return Err(CanvasError::Invalid("connection Undo was not exact".into()));
    }
    session.apply(session.prepare_undo()?)?;
    if session.document().raw_json() != source {
        return Err(CanvasError::Invalid("port setup Undo was not exact".into()));
    }
    println!("Named endpoints survived save/reparse; both Undo steps restored exact bytes.");
    Ok(())
}
