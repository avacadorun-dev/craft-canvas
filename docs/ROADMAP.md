# CraftCanvas roadmap

Updated2026-11-09. **One active library issue, strict numerical order.** Current implementation is the qualified partial foundation99db516; no production/full-parity claim. This roadmap is the execution authority. The [production plan](PRODUCTION_PLAN.md) defines architecture/readiness, [RF inventory](REACT_FLOW_PARITY.md) maps the exact source API and behaviors, and [consumer contract](WRITERCRAFT_CONSUMER.md) preserves every original Studio B01–B26 requirement.

## Two deliverables

1. **WriterCraft-ready** after CC-12: qualify every generic primitive currently needed by the original Studio Canvas and provide an exact integration handoff. This milestone does not require the downstream app to finish integration before it can start.
2. **Complete React Flow production parity** after CC-14: finish all remaining standalone capabilities and accept one exact production candidate. The first checkpoint does not waive the second.

A separate WriterCraft agent is authorized to complete a coherent non-Canvas alpha.3 in parallel. It owns app behavior/release and does not edit this engine. It resumes Canvas integration only after the exact qualified handoff. Current preview and published app releases remain untouched.

## Ordered issues

| Step | Issue | Result | State |
|---|---|---|---|
| CC-01 | [#1](https://github.com/avacadorun-dev/craft-canvas/issues/1) | Full source contract, native equivalents and independent omission review | Complete — [9da1c24](https://github.com/avacadorun-dev/craft-canvas/commit/9da1c24f08efd2742239996401da3f590506d7f0) |
| CC-02 | [#2](https://github.com/avacadorun-dev/craft-canvas/issues/2) | Named/dynamic ports, validation and compatible legacy codec | Complete — [99db516](https://github.com/avacadorun-dev/craft-canvas/commit/99db5166f287831c96da7ba6630a2bb4d620ee4e) |
| CC-03 | [#3](https://github.com/avacadorun-dev/craft-canvas/issues/3) | Direct owned creation/reconnection with cancellation | Active |
| CC-04 | [#4](https://github.com/avacadorun-dev/craft-canvas/issues/4) | Selection, movement, resizing, rotation, groups and host drop insertion | Queued |
| CC-05 | [#5](https://github.com/avacadorun-dev/craft-canvas/issues/5) | Custom node/edge/handle/label native composition | Queued |
| CC-06 | [#6](https://github.com/avacadorun-dev/craft-canvas/issues/6) | Camera navigation, minimap, controls and overlays | Queued |
| CC-07 | [#7](https://github.com/avacadorun-dev/craft-canvas/issues/7) | Drawing, shapes and frame primitives | Queued |
| CC-08 | [#8](https://github.com/avacadorun-dev/craft-canvas/issues/8) | Structured clipboard, actual required layouts and export primitives | Queued |
| CC-09 | [#9](https://github.com/avacadorun-dev/craft-canvas/issues/9) | Keyboard, accessibility and native input ownership | Queued |
| CC-10 | [#10](https://github.com/avacadorun-dev/craft-canvas/issues/10) | Measured performance, supported capacity and reliability | Queued |
| CC-11 | [#11](https://github.com/avacadorun-dev/craft-canvas/issues/11) | Useful verified Cargo packages, docs and release process | Queued |
| CC-12 | [#12](https://github.com/avacadorun-dev/craft-canvas/issues/12) | Independently qualified engine handoff to WriterCraft | Queued |
| CC-13 | [#13](https://github.com/avacadorun-dev/craft-canvas/issues/13) | Remaining complete React Flow APIs and composition behaviors | Queued |
| CC-14 | [#14](https://github.com/avacadorun-dev/craft-canvas/issues/14) | Full native parity and production release acceptance | Queued |

## Execution and closure rules

- Only the first unfinished issue is active. Finish its specification, implementation, relevant hosted checks, required native observations and independent review before moving to the next. A blocked issue stays active while its blocker is resolved; no silent leap to easier work.
- Reconcile and reuse existing behavior before editing. Scope each delegated task to files, expected result/evidence and stop condition. Root owns integration, commits/publication and hosted dispatch.
- Closure records exact SHA, tests/builds, native observations or an explicit documentation-only disposition, reviewer outcome and limitations. A missing required criterion keeps the issue open. Do not use automatic close keywords or count source-only implementation as acceptance.
- Lean verification uses existing checks, representative cross-feature native journeys and regressions for actual defects. Inventory completeness is mandatory; a test per API property or exhaustive combination matrix is not.
- All consumer-required primitives must be delivered before CC-12; mixed RF rows may retain broader library options in CC-13 only where the consumer map explicitly does not need them. CC-04 includes B07 external/project drop geometry and atomic insertion; CC-08 requires usable layout behavior/providers, not merely a callback scaffold. No baseline function is silently deferred by classification.
- Subsequent scope corrections update the issue, inventory, feedback/architecture and reviewer criteria before reacceptance. Preserve history rather than rewriting failed evidence as success.

## Planning estimate

Preparation is this CC-01 deliverable. The WriterCraft-ready work is eleven subsequent verified issues, not one connector patch. Initial implementation planning is **several weeks of engineering work**, with low confidence until the named-port and direct-interaction milestones expose integration costs. This is an effort estimate, not a calendar promise or reason to wait for upstream. Re-estimate after CC-03 using actual implementation/hosted/native review results, and report remaining issues rather than an invented percentage. Full standalone parity follows as its own explicit checkpoint.

## Package status

README and crate READMEs describe real current Git consumption. Registry names were available at the2026-11-09 check; **no crates.io registration/publication has occurred**. CC-11 implements and qualifies the [release procedure](RELEASING.md), including account setup, packaged licenses and dependency-order publication. A meaningful prerelease may precede full parity; its scope must remain honest.
