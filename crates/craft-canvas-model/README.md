# craft-canvas-model

The GUI-independent graph, geometry and editing core of [CraftCanvas](https://github.com/avacadorun-dev/craft-canvas). Use it in native graph editors, headless tools or applications that need lossless JSON Canvas data and explicit control over when an edit becomes authoritative.

The library provides a canonical document and guarded commands. It performs no storage, clipboard, network or media I/O. Resource references remain inert data until the host authorizes their use.

## Status and installation

Current baseline: **Rust 1.95**, edition 2024. This crate has no egui dependency. The planned crates.io name is `craft-canvas-model`; **it has not yet been published there**. Pin the reviewed source revision:

```toml
[dependencies]
craft-canvas-model = { git = "https://github.com/avacadorun-dev/craft-canvas", rev = "6b68eb4e84aa41b713a148e6a456048f3c1168f9" }
```

Full native React Flow functional parity is the project goal, not a claim about this baseline. Named/dynamic ports, richer graph relationships and the complete graph API remain under development. Start with the [ordered roadmap](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/ROADMAP.md), [production plan](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/PRODUCTION_PLAN.md) and [parity inventory](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/REACT_FLOW_PARITY.md); moving documentation does not change the pinned code.

## Current capabilities

- Bounded text/file/link/group nodes and edges, with future node kinds and unknown envelopes preserved.
- Exact no-edit bytes, stable IDs, array order and opaque numeric lexemes; duplicate-key rejection before interpretation.
- Atomic insertion, geometry/text/field edits, deletion, duplication with explicit ID mapping, reordering, labels and exact endpoint reconnection.
- Private in-process session identity, immutable prepared candidates and guarded Apply/Undo/Redo.
- Configurable data/work/history bounds, cooperative preparation cancellation and explicit recoverable errors.
- f64 camera/rectangle geometry, connector routes and bounded flattening/hit helpers.

Preserved future data is not silently given invented edit semantics. `CanvasDialect` supplies node policies, endpoint eligibility and application validation. The current side-based endpoint type does not yet represent several named handles on one side.

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

## Persistent host contract

In a persistent application, inspect `PreparedChange::document()` and checkpoint it against the host's expected saved revision **before** calling `CanvasSession::apply`. Save failure leaves the existing session unchanged; retain the candidate/draft for recovery. Serialize changes and acknowledgements so a successful checkpoint applies to its intended session. Undo and Redo follow the same prepare/checkpoint/apply sequence.

`ContentIdentity` refuses stale or foreign session requests. It cannot be deserialized as a mutation proof and does not grant resource permission. Caller-owned IDs and explicit `IdMapping` keep identity allocation outside the library. The engine never reports a durable save on the host's behalf.

`CanvasLimits::default()` currently preserves the Studio profile: 12 MiB snapshots, 5,000 nodes, 10,000 edges, integral stored geometry, minimum dimension 40, 200 history steps and 32 MiB historical source budget. These are configurable constraints, not measured throughput promises. `HistoryUsage` distinguishes encoded source/payload accounting from allocator or process-memory claims. Heavy preparation can use `prepare_with_control` on a host worker.

## Evidence and development

Hosted workspace tests, formatting and strict Clippy run on Linux, macOS and Windows. They qualify the model at their exact revision; they do not prove native GUI behavior or application storage. Keep public fixtures synthetic. Source contracts live in [architecture](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/architecture.md), [verification policy](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/verification.md) and [OpenSpec changes](https://github.com/avacadorun-dev/craft-canvas/tree/feature/native-canvas-foundation/openspec/changes).

## License and provenance

MIT, with third-party notices retained. Connector mathematics includes attributed XYFlow adaptations; see [upstream provenance](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/upstream.md) and [license notices](https://github.com/avacadorun-dev/craft-canvas/tree/feature/native-canvas-foundation/LICENSES). No JavaScript runtime is included.
