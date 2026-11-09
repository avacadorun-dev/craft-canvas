# Production CraftCanvas

Decision date: 2026-11-09. Status: active preparation; no production or full-parity acceptance. The user explicitly prioritizes this standalone library, then its use in WriterCraft to complete original Writers Studio Canvas. A separately coordinated WriterCraft agent may deliver the explicitly authorized non-Canvas alpha.3; it does not edit this engine or integrate unqualified work. Work continues from qualified `6b68eb4e84aa41b713a148e6a456048f3c1168f9`, not from a new engine.

## Definition of the outcome

CraftCanvas must provide the complete observable capability of the pinned React Flow baseline through a native Rust API, with safe composition of real text/media controls. It must also supply the generic primitives required by the original Studio board. WriterCraft retains its domain workflows. Passing one outcome does not imply the other:

1. **Library:** every required [React Flow capability](REACT_FLOW_PARITY.md) has a native equivalent, a reachable standalone example, relevant semantic/input evidence and independent acceptance.
2. **Consumer:** every [Studio B01–B26 workflow](WRITERCRAFT_CONSUMER.md) is reachable and accepted in the compiled WriterCraft candidate, using an exact qualified engine revision.

Parity means functional capability, not copying React syntax or depending on a browser. Component hooks, DOM measurement, CSS interaction exclusions, portals and accessibility announcements require explicit native equivalents. Official examples, integration recipes and Pro demonstrations remain visible in the inventory; classifying a feature does not waive it. No omitted capability can be silently marked not applicable. Upstream defects are documented and corrected rather than reproduced.

Initial reference: [`@xyflow/react`12.12.1 / `@xyflow/system`0.0.84](https://github.com/xyflow/xyflow/tree/1bc405cd113618c17fd181b931a9fe573bec9fee), published 2026-11-02. Freeze source identity for qualification; assess later upstream changes explicitly rather than moving the target while accepting a candidate. The source Studio contract is frozen separately by its existing file hashes. CraftCanvas owns implementation and maintenance: no milestone waits for egui-xyflow or another project's next commit.

## Preserve and build on

- One canonical document, opaque in-process session identity, exact-base preparation, immutable candidates and guarded Apply/Undo/Redo.
- Original no-edit bytes, stable IDs, node/edge array order, unknown envelopes and numeric lexemes. Never reconstruct unknown data through a renderer model.
- f64 world geometry with local-origin screen projection. The existing Studio profile retains its source capacities and geometry rules.
- Synthetic public fixtures only; source application and private projects remain read-only. Host resource capabilities remain explicit.
- Existing native editor focus/IME and pointer/keyboard ownership work, qualified connector geometry and provenance.
- Existing user preview and app pins remain unchanged during library preparation. Retained app/capture/portability work is not discarded or folded into engine commits.

Current foundations are partial. CC-02 qualifies named/dynamic port data, pure validation and guarded model commands. Native named-port geometry/gestures, complete custom rendering and interaction hooks, parent relationships, viewport controls, touch and complete accessibility remain pending. Current library tests are engineering evidence, not production certification.

## Delivery order and stopping conditions

The issue sequence in [ROADMAP.md](ROADMAP.md) is authoritative. One library issue is implemented at a time. Specification and independent source review precede implementation. Root owns integration, commits, publication and hosted dispatch; delegates own named files and stop at a bounded result. Each milestone finishes its relevant checks and actual native evidence before the next implementation milestone starts. Existing tests plus focused native journeys are preferred over a combinatorial test matrix.

The first checkpoint is **WriterCraft-ready**, after CC-12: every generic primitive needed by the original consumer is qualified and delivered with an exact integration handoff. It does not claim complete React Flow parity. CC-13 and CC-14 complete and accept the remaining full standalone goal. Actual WriterCraft B01–B26 integration is independently accepted by the application workstream after handoff; it is not a circular prerequisite for engine qualification.

| Preparation / delivery group | Strict roadmap steps | Result |
|---|---|---|
| Source-backed preparation | CC-01 | Reviewed RF inventory, all B01–B26 boundaries, architecture, roadmap/issues, honest standalone documentation and app handoff. |
| Graph/input foundation | CC-02 → CC-03 | Stable named/dynamic ports, compatible codec/validation and controlled direct creation/reconnection. |
| Consumer primitives | CC-04 → CC-08 | Complete manipulation/groups, native composition, viewport, drawing/shapes/frames and clipboard/layout/export primitives. |
| Consumer qualification | CC-09 → CC-12 | Native input/accessibility, measured capacity/reliability, real Cargo packages and independently qualified WriterCraft-ready handoff. |
| Full standalone parity | CC-13 → CC-14 | Remaining RF API/pattern coverage and exact production acceptance. |

Each issue has bounded acceptance and file ownership; larger issues use focused working commits, never competing implementation. New omissions enter the relevant issue before closure. Complete an issue's checks/review before the next starts. An early engineering package is not a production label. Release version/tag notes must describe the precise qualified scope.

## Production readiness criteria

These are required outcomes, not a new exhaustive test matrix. Reuse existing checks and capture only the native workflows needed to establish each changed capability. Add regressions for actual defects; sample representative interaction combinations and expand when evidence exposes a gap.

| Gate | Required evidence |
|---|---|
| Scope completeness | Independently reviewed RF inventory and B01–B26 boundary; every row maps to specifications, reachable API/example and evidence. No source-only row marked accepted. |
| Data and authority | Existing exact bytes/unknown fields/history tests pass; named ports, parent transforms and custom data round-trip semantically; stale/failed/cancelled operations retain original data. Public requests never deserialize mutation authority. |
| API and extensibility | Documented types and failure modes, controlled graph/viewport lifecycle, custom native controls/routes/handles, stable identity, complete query/event contract, migration notes for any public API break. Standalone minimal consumer compiles without WriterCraft dependencies. |
| Native usability | Independent actual compiled interactions cover create/edit/reconnect/select/move/resize/group/navigation with keyboard and pointer; touch and native accessibility have explicit platform evidence. Embedded text composition/selection/Undo and media controls retain input. Interactive affordances are discoverable and unclipped. |
| Performance and capacity | Release-build metrics on named hardware/compiler/source; declared workloads and view states, p50/p95/p99 frame/input time, peak memory and bounded history. All existing source-valid capacities remain supported; a benchmark miss is not permission to reduce them. |
| Reliability | Repeated create/delete/navigation cycles do not retain unbounded resources; invalid geometry, cyclic parenting, removed ports, stale bases and cancellation fail actionably without partial changes. No uncontrolled graph mutation or panic on valid host input. |
| Distribution and maintenance | Locked tests/fmt/strict Clippy/docs on supported compiler, reviewed dependencies and notices, reproducible package/source identity, changelog/API compatibility policy and useful examples. No unchecked platform-readiness claim from cross-compilation alone. |
| Downstream consumer acceptance (application workstream, after CC-12) | Exact qualified pin adopted by WriterCraft, complete original workflows independently accepted there. This remains required for the overall product objective, but is not a prerequisite that would prevent the engine handoff. A standalone showcase cannot accept persistence, import or manuscript relationships. |

Initial engineering performance targets (not measured results): 60 Hz interaction budget with p95 frame work ≤16.7 ms and p99 ≤33.3 ms for a mixed 1,000-node/2,000-edge board with 100 visible nodes/200 visible edges, at 1440×900 logical points. Also measure 5,000 nodes/10,000 edges at close and fit-all zoom, offscreen culling, route update during drag and embedded input; record visible counts and body complexity. Retain the source snapshot/history bounds (12 MiB snapshot, 32 MiB historical encoded sources under existing defaults). Track actual process memory separately; encoded bytes are not allocator/RSS claims. Set a measured capacity-specific interaction envelope during CC-01/CC-02 preparation before accepting CC-10/CC-14; do not invent achieved numbers or hide the fit-all case.

The first native product target remains macOS ARM64 to match the real consumer. Portable Rust model/adapter tests run on macOS, Windows and Linux. Windows/Linux native input, accessibility and graphics remain unaccepted until exercised there; platform support statements must distinguish portable library checks from actual native readiness. Trackpad pinch and injected touch events do not establish direct touchscreen behavior; record an actual device/platform before claiming general touch support. Shared behavior stays portable; no platform-only shortcut can claim general parity.

## Current checkpoint and immediate next action

CC-01 completed at9da1c24 with independent preparation review and hosted qualification42626580148. CC-02 completed at99db516 with independent API review and hosted qualification42633082977. CC-03 is active. Reference research ruled out unchanged adoption of the inspected Rust alternatives; the chosen direction is a maintained native implementation on the existing CraftCanvas authority, with selective attributed reuse. CC-01 changed preparation only; CC-02 added the named-port model without new dependencies. CC-03 must use that model rather than the earlier four-side-only gesture proposal.

Implement CC-03 against the accepted named-port model and exact connection validation contract; specify owned gesture/events, measurement invalidation and cancellation before modifying native input. Reuse the exact current foundation; do not restart the implementation. Keep engine production and WriterCraft full Canvas acceptance explicitly pending throughout.
