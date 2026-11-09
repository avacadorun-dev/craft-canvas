# CraftCanvas

**A native Rust library for interactive node graphs and canvas applications.**

CraftCanvas combines a lossless graph model, guarded editing commands and an egui interface. It is designed for applications that need custom node content, precise board geometry and control over how changes become saved data. The model also works without a GUI. Applications supply their own storage, resource access and domain rules.

The development goal is **full functional React Flow parity in native Rust**, with a qualified library supporting complete original Studio Canvas integration in WriterCraft. CraftCanvas is a standalone library; it does not depend on WriterCraft, React, a WebView or a browser runtime. Full parity and production qualification are **work in progress**, not properties of the current candidate. Start with the [ordered roadmap](docs/ROADMAP.md), [production plan](docs/PRODUCTION_PLAN.md) and [versioned capability inventory](docs/REACT_FLOW_PARITY.md).

## Packages

| Package | Purpose |
|---|---|
| [`craft-canvas-model`](crates/craft-canvas-model/README.md) | Bounded JSON Canvas parsing and preservation, f64 geometry, atomic commands, exact-base sessions and Undo/Redo. No GUI dependency. |
| [`craft-canvas-egui`](crates/craft-canvas-egui/README.md) | Native viewport, selection and geometry interaction, connector rendering/hit testing, host body hooks and retained native text-card input. Emits requests to the host. |

The baseline uses **Rust 1.95**, edition 2024, and **egui 0.36.2**. The planned crates.io package names are `craft-canvas-model` and `craft-canvas-egui`; **these packages are not yet published on crates.io**. Use a reviewed Git revision while the public API develops.

## What is implemented

- Standard text, file, link and group records, with unknown node kinds and extension data retained. Preserving an unknown record does not imply an editor for it.
- Exact no-edit source bytes; stable IDs and node/edge order; unknown JSON fields and numeric lexemes preserved across supported edits. Duplicate keys are rejected before interpretation.
- Explicit insertion, text/field edits, geometry changes, duplication/remapping, removal, reordering, labels and endpoint reconnection commands. Reconnection currently exists in the model; it is not yet a complete native drag workflow.
- Stable named ports in the model, multiple ports per side, strict/loose connection rules, dynamic availability and original-edge reconnect policy. The optional codec preserves legacy/unknown data and supports atomic port removal with edge repair.
- Immutable prepared changes checked against a private session identity, compensating Undo/Redo and bounded history/work accounting.
- f64 world/camera geometry with origin subtraction before screen conversion.
- Native node/edge selection, movement, single-corner resize, marquee, pan/zoom/fit and routed connectors with arrow markers and shared paint/hit geometry.
- Native body hooks with pointer/keyboard ownership. A retained text-card editor supports exact-base drafts, insertion acknowledgement and native input/IME integration hooks.

The current API does **not yet establish native named-port painting/measurement, direct connection dragging/reconnection, complete custom graph rendering, parent/subflow transforms, viewport controls, touch interaction or graph accessibility**. Source compatibility and actual input acceptance remain explicit gates. Drawing, layout and export primitives also need further work; application media, clipboard I/O and project workflows belong to the host. The [consumer contract](docs/WRITERCRAFT_CONSUMER.md) records that boundary without reducing required features.

## Use the qualified source baseline

This pins the existing engine baseline rather than a moving branch. Reproducible source selection does not establish production or application UX acceptance.

```toml
[dependencies]
craft-canvas-model = { git = "https://github.com/avacadorun-dev/craft-canvas", rev = "99db5166f287831c96da7ba6630a2bb4d620ee4e" }
craft-canvas-egui = { git = "https://github.com/avacadorun-dev/craft-canvas", rev = "99db5166f287831c96da7ba6630a2bb4d620ee4e" }
egui = "=0.36.2"
```

Headless clients need only `craft-canvas-model`. Use a compatible egui version throughout the native host; UI types from different egui versions are not interchangeable.

### Edit a document in memory

This example uses the existing public API. It deliberately has no persistence: `apply` acknowledges an in-memory change, not a saved file.

```rust
use craft_canvas_model::{
    CanvasCommand, CanvasDocument, CanvasLimits, CanvasSession,
    CanvasTransaction, StandardDialect,
};

fn main() -> craft_canvas_model::Result<()> {
    let source = r#"{"nodes":[{"id":"note","type":"text","x":0,"y":0,"width":240,"height":120,"text":"First draft"}],"edges":[]}"#;
    let limits = CanvasLimits::default();
    let document = CanvasDocument::parse(source, &limits)?;
    let mut session = CanvasSession::new(document, limits)?;
    let request = CanvasTransaction::new(
        session.identity(),
        vec![CanvasCommand::EditText {
            node_id: "note".into(),
            expected: "First draft".into(),
            text: "A revised note".into(),
        }],
    );
    let candidate = session.prepare(request, &StandardDialect)?;
    assert_eq!(session.document().raw_json(), source);
    session.apply(candidate)?;

    let undo = session.prepare_undo()?;
    session.apply(undo)?;
    assert_eq!(session.document().raw_json(), source);
    Ok(())
}
```

### Mount the native viewport

Retain `CanvasView` and `CanvasSession` in application state. Call this from the host's egui UI callback and dispatch the returned output through that host.

```rust
use craft_canvas_egui::{CanvasView, CanvasViewConfig, CanvasViewOutput, PlainTextBody};
use craft_canvas_model::{CanvasSession, StandardDialect};

fn show_board(
    ui: &mut egui::Ui,
    view: &mut CanvasView,
    session: &CanvasSession,
) -> CanvasViewOutput {
    let config = CanvasViewConfig::new(egui::Id::new("example-board"), ui);
    view.show(ui, session, &StandardDialect, &mut PlainTextBody, &config)
}
```

`PlainTextBody` is an inert example renderer. Implement `CanvasViewHost` for richer bodies, provide localized `CanvasViewLabels`, and respect body/host input ownership. Handle `output.errors` and every relevant `output.intents` variant; returning a frame is not a save acknowledgement. See the [egui integration guide](crates/craft-canvas-egui/README.md).

## One authority, explicit acknowledgement

The native view is a disposable projection of one `CanvasSession`. It emits `CanvasIntent` values rather than mutating a parallel graph. A persistent host processes a transaction as follows:

1. Capture the request's exact session base and the host's current durable revision.
2. Call `prepare` or `prepare_with_control`; inspect the immutable candidate and validate domain/resource policy.
3. Checkpoint `candidate.document()` through the host's guarded storage path. Refuse stale or failed saves without overwriting the canonical graph; retain the candidate/draft for deliberate recovery.
4. Call `apply` only after the required checkpoint succeeds. Apply rechecks the private session base. Use the same durable path for prepared Undo/Redo.

Serialize concurrent host changes and acknowledgements so a successful checkpoint cannot be acknowledged to the wrong local session. `ContentIdentity` is an in-process freshness guard, not a project permission or database revision. File and URL strings grant no filesystem/network access. IDs for inserted/duplicated records are allocated explicitly by the caller.

Known geometry and encoded snapshots/history have configurable bounds. Current defaults preserve the Studio profile; limits are not universal graph semantics or measured performance claims. Unknown data never passes through a lossy renderer model. See [architecture](docs/architecture.md), [product contract](docs/PRD.md) and [verification policy](docs/verification.md).

## Qualification and platform status

Qualified source `99db516` passed [hosted run42633082977](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42633082977), including104 workspace tests and the headless named-port example. Hosted qualification checks formatting, locked workspace tests and strict Clippy on **Linux, macOS and Windows**. Those checks establish portable engineering evidence for the tested source. They do not certify OS-level input, graphics, accessibility or every host integration.

The first native product qualification target is **macOS ARM64**. Full standalone production/native acceptance remains pending, including Windows/Linux native interaction. Existing input and connector evidence retains its precise scope; the current interface is not advertised as complete React Flow behavior. See the [production gates](docs/PRODUCTION_PLAN.md).

## Relationship to React Flow and other libraries

React Flow is the functional reference; CraftCanvas provides native Rust equivalents rather than React components or DOM emulation. Public capabilities and documented patterns are tracked in [React Flow parity](docs/REACT_FLOW_PARITY.md). Current generic graph libraries were evaluated as possible dependencies; the chosen direction is maintained development on this model with selective, attributed reuse, not waiting for another library to complete the work.

Some connector mathematics is adapted from MIT-licensed XYFlow source. Exact source/version/hash and source-bug corrections are recorded in [upstream provenance](docs/upstream.md), with [XYFlow's notice](LICENSES/XYFlow-MIT.txt). React Flow is not a runtime dependency, and this is not an official XYFlow project. Preservation and host authority remain CraftCanvas responsibilities.

## Roadmap and contributing

The [ordered roadmap](docs/ROADMAP.md) is the entry point for work and linked issues. It distinguishes the WriterCraft-ready checkpoint from complete React Flow parity; reaching the former does not waive the latter. The [production plan](docs/PRODUCTION_PLAN.md) defines the qualification criteria, and the [consumer map](docs/WRITERCRAFT_CONSUMER.md) retains every original B01–B26 workflow. API changes must preserve compatibility contracts and record migrations; a library demonstration does not accept a downstream product.

Read [AGENTS.md](AGENTS.md), [architecture](docs/architecture.md) and relevant [OpenSpec changes](openspec/changes) before behavior changes. Keep public fixtures synthetic and contributions focused. The current workflow runs Cargo qualification on hosted CI:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Check public API examples and packages on the exact candidate before publication. Native interface changes additionally need actual compiled interaction evidence and independent review. No benchmark, crate release or full-parity result is implied by this README.

## License

CraftCanvas is [MIT licensed](LICENSE). Reused source and dependencies retain their own notices; see [upstream provenance](docs/upstream.md) and [dependency policy](docs/dependencies.md).
