# craft-canvas-model

The GUI-independent graph, geometry and editing core of [CraftCanvas](https://github.com/avacadorun-dev/craft-canvas). Use it in native graph editors, headless tools or applications that need lossless JSON Canvas data and explicit control over when an edit becomes authoritative.

The library provides a canonical document and guarded commands. It performs no storage, clipboard, network or media I/O. Resource references remain inert data until the host authorizes their use.

## Status and installation

Current baseline: **Rust 1.95**, edition 2024. This crate has no egui dependency. The planned crates.io name is `craft-canvas-model`; **it has not yet been published there**. Pin the reviewed source revision:

```toml
[dependencies]
craft-canvas-model = { git = "https://github.com/avacadorun-dev/craft-canvas", rev = "d2bdaeca02ca028693222cf16153ff7c34c7b874" }
```

Full native React Flow functional parity is the project goal, not a claim about this baseline. The named-port model is qualified in this revision; richer graph relationships and the complete graph API remain under development. Start with the [ordered roadmap](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/ROADMAP.md), [production plan](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/PRODUCTION_PLAN.md) and [parity inventory](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/REACT_FLOW_PARITY.md); moving documentation does not change the pinned code.

## Current capabilities

- Bounded text/file/link/group nodes and edges, with future node kinds and unknown envelopes preserved.
- Exact no-edit bytes, stable IDs, array order and opaque numeric lexemes; duplicate-key rejection before interpretation.
- Atomic insertion, geometry/text/field edits, deletion, duplication with explicit ID mapping, reordering, labels and exact endpoint reconnection.
- Private in-process session identity, immutable prepared candidates and guarded Apply/Undo/Redo.
- Configurable data/work/history bounds, cooperative preparation cancellation and explicit recoverable errors.
- f64 camera/rectangle geometry, connector routes and bounded flattening/hit helpers.

Preserved future data is not silently given invented edit semantics. `CanvasDialect` supplies node policies, endpoint eligibility and application validation. `PortEndpoint` adds multiple named ports without changing the existing side-based `ConnectorEndpoint`.

## Minimal in-memory edit

```rust
use craft_canvas_model::{
    CanvasCommand, CanvasDocument, CanvasLimits, CanvasSession,
    CanvasTransaction, StandardDialect, WorldPoint,
};

fn main() -> craft_canvas_model::Result<()> {
    let source = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":200,"height":120,"text":"Note"}],"edges":[]}"#;
    let limits = CanvasLimits::default();
    let document = CanvasDocument::parse(source, &limits)?;
    let mut session = CanvasSession::new(document, limits)?;
    let request = CanvasTransaction::new(
        session.identity(),
        vec![CanvasCommand::Translate {
            node_ids: vec!["a".into()],
            delta: WorldPoint::new(24.0, 12.0)?,
        }],
    );
    let candidate = session.prepare(request, &StandardDialect)?;
    assert_eq!(session.document().raw_json(), source);
    session.apply(candidate)?; // This example has no persistent storage.

    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), source);
    Ok(())
}
```

## Interaction policy in the CC-04 development branch

The active CC-04 work adds defaulted `CanvasDialect::node_interaction` and `edge_interaction` hooks. Node policies independently choose selectability, forward deletion, wrapper focus and visibility; edge policies choose selectability, forward deletion and visibility. Existing dialects retain their defaults, including unknown records. `NodePolicy` continues to control movement, resize and spatial-group/frame behavior. No stored field automatically becomes a permission or visibility flag.

`RemoveSelection` checks both original and current candidate records before removing any selected node or selected/incident edge. A protected incident edge refuses the entire transaction; earlier field edits cannot bypass original protection. Group removal retains enclosed members. Exact historical Undo/Redo and prepared Apply keep their existing identity contract; persistent hosts still own authorization and policy-change serialization.

These additions require the development revision recorded in [CC-04 progress](../../docs/CC04_PROGRESS.md); the qualified source pin above predates them. Full issue/native acceptance remains pending.

## Persistent host contract

In a persistent application, inspect `PreparedChange::document()` and checkpoint it against the host's expected saved revision **before** calling `CanvasSession::apply`. Save failure leaves the existing session unchanged; retain the candidate/draft for recovery. Serialize changes and acknowledgements so a successful checkpoint applies to its intended session. Undo and Redo follow the same prepare/checkpoint/apply sequence.

`ContentIdentity` refuses stale or foreign session requests. It cannot be deserialized as a mutation proof and does not grant resource permission. Caller-owned IDs and explicit `IdMapping` keep identity allocation outside the library. The engine never reports a durable save on the host's behalf.

`CanvasLimits::default()` currently preserves the Studio profile: 12 MiB snapshots, 5,000 nodes, 10,000 edges, integral stored geometry, minimum dimension 40, 200 history steps and 32 MiB historical source budget. These are configurable constraints, not measured throughput promises. `HistoryUsage` distinguishes encoded source/payload accounting from allocator or process-memory claims. Heavy preparation can use `prepare_with_control` on a host worker.

## Evidence and development

Hosted workspace tests, formatting and strict Clippy run on Linux, macOS and Windows. They qualify the model at their exact revision; they do not prove native GUI behavior or application storage. Keep public fixtures synthetic. Source contracts live in [architecture](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/architecture.md), [verification policy](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/verification.md) and [OpenSpec changes](https://github.com/avacadorun-dev/craft-canvas/tree/feature/native-canvas-foundation/openspec/changes).

## License and provenance

MIT, with third-party notices retained. Connector mathematics includes attributed XYFlow adaptations; see [upstream provenance](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/upstream.md) and [license notices](https://github.com/avacadorun-dev/craft-canvas/tree/feature/native-canvas-foundation/LICENSES). No JavaScript runtime is included.

## Stable named ports

The qualified model provides `SetPorts`, `PortEndpoint`, pure `validate_connection`, `EdgeRecord::connect_ports` and guarded `ConnectPorts`/`ReconnectPort`. These APIs are included in the qualified Git pin above; native named-port gestures remain a later milestone. The [headless example](examples/named_ports.rs) authors two ports on one side, previews a connection, prepares/applies it, reparses saved data and checks exact Undo. Its hosted qualification command is `cargo run -p craft-canvas-model --example named_ports --locked`; running it provides model evidence, not native interaction or production acceptance.

The optional `craftCanvas` version1 extension stores node-local `(kind,id)` port keys, finite local anchor geometry and edge bindings. `connectable`, `start` and `end` default true; omitted edge bindings retain legacy side semantics, and omitted reconnect policy inherits the host default. Strict mode requires normalized source→target direction; actual initiation controls start/end availability. See the [exact schema and API contract](../../openspec/changes/named-ports-validation/design.md).

Unknown members and unchanged omitted/default values remain preserved. Unsupported or malformed extensions refuse operations that require their interpretation; unrelated supported edits remain possible. Removing an attached port requires repairing or removing its incident bindings in the same atomic transaction. Preview success never grants Apply or durable-save authority; final preparation rechecks the complete candidate and original reconnect policy.
