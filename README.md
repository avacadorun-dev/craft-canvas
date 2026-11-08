<div align="center">

# CraftCanvas

**A small native Rust foundation for editable JSON Canvas boards.**

Opaque data preservation · guarded transactions · precise camera geometry · egui

</div>

CraftCanvas separates reusable board mechanics from application ownership and persistence. `craft-canvas-model` provides bounded parsing, atomic graph changes and guarded session history. `craft-canvas-egui` provides a native interaction adapter that emits the same semantic commands used by headless clients.

This is an early engineering foundation for WriterCraft. It is **not yet a complete Canvas editor**, a media engine or a claim of WriterCraft alpha.2 parity. Native interaction and host integration must be independently qualified before release.

## Boundaries

- Four standard JSON Canvas node kinds and preserved future envelopes.
- Complete no-edit byte retention, stable IDs/order and unknown arbitrary-precision JSON.
- Explicit prepare/apply with private change authority and stale-base refusal.
- One gesture per undo unit, bounded history and relative-origin f64 camera math.
- Host-owned labels, resources, clipboard, storage, permissions and media rendering.

See [the product contract](docs/PRD.md), [architecture](docs/architecture.md), [active specification](openspec/changes/native-canvas-foundation/specs/canvas-foundation/spec.md) and [qualification policy](docs/verification.md).

## Development

```sh
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
```

The current development task runs Cargo only on hosted CI. The resolved lock is reviewed and committed before qualification; a generated lock or compiler pass is not a release.

## License

New source is [MIT](LICENSE). Direct dependencies and any attributed upstream work keep their original licenses; see [dependency policy](docs/dependencies.md).
