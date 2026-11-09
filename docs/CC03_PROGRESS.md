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
