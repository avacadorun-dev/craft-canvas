# CC-03 direct native connection progress

Updated 2026-11-09. Issue [#3](https://github.com/avacadorun-dev/craft-canvas/issues/3) is the only active library issue. CC-02's qualified source remains `99db5166f287831c96da7ba6630a2bb4d620ee4e`; this candidate has not yet passed hosted or native qualification. No consumer pin changes or app preview changes are included.

## Integrated candidate

The [OpenSpec change](../openspec/changes/direct-connection-gestures/design.md) covers shared named/legacy frame geometry, handle drag creation, either-end reconnection, typed lifecycle events, terminal input ownership and exact pending host acknowledgement. The isolated native showcase exercises the real adapter against synthetic data with embedded native controls and explicit delayed/rejected in-memory acknowledgement.

Root owns view integration, public exports, documentation and hosted dispatch. Bounded implementation agents supplied geometry, controller and showcase files; a separate agent reviewed the same issue without edits. Source review found five concrete defects: co-located legacy/named no-op, coincident self-loop grip reachability, stationary overlay takeover, Space-pan precedence and touch input depending on synthesized mouse coordinates. Corrections and focused regressions are integrated. Follow-up review refined grip separation for closely adjacent nodes and Space-pan precedence for a complete press/release batch. Direct formatting and Python syntax checks pass; source review alone is not native acceptance.

## Required remaining evidence

- Commit the integrated source; resolve and inspect the exact changed lock on GitHub.
- Pass formatting, existing and targeted tests, headless example and strict Clippy on the exact committed candidate across Linux/macOS/Windows.
- Build and hash the isolated macOS ARM64 showcase; run actual graphics readback and natural OS interactions with honest input provenance.
- Independently accept creation, both-end revision/no-op, interruption/recovery and embedded-control journeys; record source, evidence hashes, reviewer outcome and limitations.
- Update the roadmap and close only when all issue criteria are verified. Full production/React Flow parity and downstream WriterCraft Canvas acceptance remain unfinished.

## Hosted dependency resolution

Candidate `4a4f94f3bacca12bf4edf7c19e51ea6a965fee33` started [run42640744601](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42640744601). The lock guard correctly refused qualification because the new standalone host was not yet in the committed lock. GitHub resolved and retained the lock; no Rust verification jobs ran at this stage.

The reviewed result contains377 packages, adding294 for the native showcase while retaining every previous name/version pair. Eframe/egui0.36.2, image0.25.10 and existing library pins are unchanged. All declared MSRVs fit Rust1.95. All external sources are registry packages; the declared licenses provide permissive alternatives, with the existing font/Unicode notices retaining their additional terms. This metadata inspection is not package-release qualification.

Exact hosted lock SHA256: `5a833ea74faaa3b70ecad110472537687a850ff0bb4fd72ac481f0d7447f44d0`. The next committed source must pass the complete locked hosted checks and native artifact review.
