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

## First locked qualification

Source `b09991368de719dcc3ea7a890a830ca38e35dda7`, [run42641033946](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42641033946): committed-lock verification passed. Linux passed all128 workspace tests and the executable headless named-port example. Strict Clippy stopped on two equivalent-form suggestions in port geometry (`collapsible_if`, `nonminimal_bool`). The follow-up combines the same short-circuit conditions and replaces negated `is_some_and` with the equivalent `is_none_or`; tests and behavior are unchanged. Other platform outcomes and exact native acceptance remain pending at this record.

Windows also passed all128 workspace tests and the headless example, then reported the same two Clippy suggestions. The queued macOS job is superseded when the reviewed correction is pushed; it supplies no native evidence.

## Qualified build and first native observations

Source `e68bbdda298dd5a4401912f3ae45b7a61c64c344` passed [run42641928999](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42641928999): committed lock, formatting, all128 workspace tests, headless named-port example, strict Clippy and standalone executable build on Linux/macOS/Windows. The macOS ARM64 bundle was downloaded separately, and its ad-hoc signature plus archive/executable hashes were verified against its hosted manifest.

- Archive SHA256: `5ddfbeb20a3cd114258620abc27ba0c0fbd20f3f0cff558ec7fb425429101b01`.
- Executable SHA256: `ce7f5d21aeb63e812a3106815c6ba9cdb0b97f9823eea1c6fbddc9796302a589`.
- Native OS pointer actions created one named connection (revision1, two edges, one Undo step), then held and rejected a second connection while the canonical counts stayed unchanged. These are partial root observations, not independent four-journey acceptance.
- The first scripted native capture retained a real baseline WGPU image but failed at the rejected-state check, with only4 of27 steps completed. It is failed evidence, not a successful capture. Investigation identified a driver readiness issue: an action target could be published while the header still rendered that action disabled, before the same frame's connection release made it available. Later state transitions in that run are not attributed to an input source without further evidence.
- Source review confirmed that the synthetic scroll body claimed any held pointer merely on hover, cancelling an incoming board drag. The host correction derives actual scroll-widget ownership and adds targeted regressions; library ownership precedence is retained.
- The user then explored the open native demo and reported that retained Conflict made the board appear button-only. [CV-02](design/USER_FEEDBACK.md) requires a clear blocked-state explanation and explicit safe recovery. The explored window is left intact; the corrected source requires a new build and native evidence.

Issue3 stays open. Full native acceptance, designer review and consumer adoption are pending.

## Rejected-demo corrections ready for hosted qualification

The follow-up keeps the engine's guarded ownership/acknowledgement contract and fixes the synthetic host: actual scroll responses own pointer input; normal mode applies immediately with direct-manipulation help and Undo/Redo; testing controls are collapsed; a pending/conflict explanation and explicit Discard pending remain reachable. Discard cancels/rejects the host-owned held candidate before clearing presentation and changes no canonical data/history.

Capture uses only enabled/unclipped header targets, a distinct Verification title and a bounded redacted upstream/scripted input trace. All27 journey steps and checks remain intact. Six focused host/header/privacy regressions were added. Independent source re-review found no blockers; exact hosted execution and fresh native acceptance remain pending.

### Pinned egui test API correction

Source `332c58c1c52f1b0c816617a33af8cc5f5d2f47a4`, [run42647241688](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42647241688), failed compilation on all three platforms: two new test harnesses called `Context::style_mut`, which does not exist in pinned egui0.36.2. No regression execution is claimed. The correction uses the public `all_styles_mut` API to disable animation for both themes. Independent review confirmed that assertions, scenarios and acceptance criteria are unchanged; direct rustfmt and whitespace checks passed. The corrected exact source needs hosted and native qualification.

### Qualified host corrections and native focus prerequisite

Source `f41a96e3b83b9841729794b27a34d2865db83c40` passed [run42648099006](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42648099006) on all three platforms: committed lock, formatting, all134 tests, executable named-port example, strict Clippy and native build. The macOS ARM64 archive hash is `1daac27930762e9accce02c141c5e8737fb2980af31a31f4b785d0be4e2e645e`; executable hash is `943f6bdae108b5afbfb24edd7955592202a42501fbb9b9a73e938f2d77f272ea`. Downloaded bytes, lock, architecture and ad-hoc signature matched the hosted manifest.

The first sandboxed native process aborted before UI startup (SIGABRT; no usable capture). A separate permitted native run retained its real baseline readback but failed: the OS reported the window unfocused from frame2, while the driver incorrectly sent its first drag at frames8–11 and waited until the frame bound. There were no root CUA actions during that run; the redacted input trace distinguishes the actual focus signal from the scripted drag. Neither attempt is accepted evidence.

The driver now waits for observed native focus, requests its own Verification window's focus once, and reports a bounded actionable failure if focus never arrives or is lost after input begins. It never forges `RawInput.focused`, resumes an interrupted scripted gesture or changes engine cancellation. Two focused regressions cover startup waiting and loss/regain interruption; all27 journey checks remain intact. Independent source review approved hosted qualification. Fresh exact-source native acceptance is still required.
