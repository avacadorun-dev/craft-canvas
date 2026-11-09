# CC-03 direct native connection progress

Updated 2026-11-09. Issue [#3](https://github.com/avacadorun-dev/craft-canvas/issues/3) is the only active library issue. CC-02's accepted source remains `99db5166f287831c96da7ba6630a2bb4d620ee4e`. CC-03 candidate `23b87d10925438750578cd44b0369dd943d0c150` passed hosted qualification and its scripted native journey, but independent ordinary interaction review requires rework for failed card movement. The later entries supersede historical pending-build paragraphs. No consumer pin changes or app preview changes are included.

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

Source `55623a1fb0bb59746198482112aa1f6a0e3df436`, [run42649588859](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42649588859), passed Linux and Windows. On macOS, two concurrent new regressions collided on their timestamp-derived temporary directory (`AlreadyExists`); the other seven showcase tests passed. Add an atomic per-process suffix to the test fixture names so timestamp resolution cannot merge concurrent fixtures. No production path, event expectation or native criterion changes. The failed run supplies no qualified macOS candidate.

### Exact build qualified; competing input invalidated the capture

Source `5d38b65d442b57ccaa1f6bbafb7210b50c248da2` passed [run42651129592](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42651129592) on macOS/Linux/Windows: all136 tests, formatting, named-port example, strict Clippy and native build. Verified macOS archive SHA256 `31aca6a789278fed58d19a866e5af89ebbe02fe6557d2d36f4ba14d384c81440`, executable `f81fd4738574b11ea4b177c9cc86357548b2dc28dd71ded85b8700d3d563ea7f`, committed lock and ad-hoc signature.

Its separate native capture acquired actual focus and created one held named connection with the canonical document unchanged. The capture then failed4/27: upstream pointer/wheel events were mixed into both scripted Reject click frames, followed later by unscripted Discard/movement and native focus loss. The redacted trace records both origins; root performed no CUA actions during the run. This is failed/mixed input evidence, not a passing journey or a user-attributed action history.

The follow-up isolates only the opt-in scripted fixture's interaction stream before the host or widgets consume it. Native focus, viewport context and WGPU readbacks remain unchanged; ordinary interactive mode remains unfiltered. Every suppressed event class is counted, and the bounded redacted trace labels suppressed, retained-native-lifecycle and injected delivery. Two focused regressions cover the actual egui click failure and lifecycle/private-payload preservation. All27 graph/history checks remain unchanged. Natural OS interaction still requires its own independent review; the scripted capture cannot accept that obligation.

Independent read-only review approved this isolation for hosted qualification, confirming the unchanged exact canonical/history/counter, Escape and screenshot-token gates. Pointer/touch interruption is intentionally outside the isolated fixture; ordinary native interruption/ownership remains a separate mandatory journey. Direct rustfmt and whitespace checks passed; new exact-source Rust execution and native capture remain pending.

### Exact native capture passed; ordinary movement remains a blocker

Source `23b87d10925438750578cd44b0369dd943d0c150` passed [run42653634875](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42653634875) on macOS/Linux/Windows: all138 tests, formatting, headless named-port example, strict Clippy and native build. The downloaded ARM64 artifact matched its source, compiler/lock, architecture, ad-hoc signature and manifest hashes: archive `e22045b30adffeef7e5335c7a014ae6ad6ed451ab039ce3be80d0dbee0b56e27`, executable `ba4dcbe43b23168ce1c1d5fe12a2a280cf37521d46f9b9b70bf3b7ac3f2f04e2`.

The first isolated capture passed11/27 before actual focus loss; preserve it as failed evidence. During the subsequently authorized quiet native window, the second capture passed27/27 in6.474603 seconds with five actual2400×1790 WGPU readbacks, no suppressed upstream input and no dropped trace entries. Final bytes exactly restore the initial fixture; revision4, one legacy edge, no pending request, Undo0/Redo2, four Begin events, two submitted, one no-op and one Escape cancellation. Two view intents, three host submissions including retry and two acknowledgements are retained. Sidecar SHA256: `a36f949c0fd5e07a26d29e91211ee27d12520bf36a72874f802126d92caf80b5`; runner result: `17dbc86d234ac58f3f4e81867fa28c2060d3b9881cd1c32c5b61a5ea3ff87e77`.

The independent reviewer then operated the ordinary unfiltered exact-source window with Testing initially collapsed. Direct creation, both-end reconnection, target-start/same-node/parallel creation, invalid-target refusal, selected-edge Delete/Undo, held-conflict explanation and safe Discard recovery were observed. Embedded typing/paste/BackSpace, scene scrolling, slider/button input and blank-board pan/return worked. Subsequent creation hit the returned painted ports. Visible revisions reached11/seven edges with no pending request. The redirected ordinary process log was empty because that launch exited and CUA opened the exact bundle separately; ordinary findings therefore use native screenshots/accessibility and visible counters, not an invented canonical trace.

**Acceptance blocker:** repeated header and unobstructed card-body drags did not move the note or increment revision, including after a successful connection and with no pending/text focus. The precise cause still requires a focused regression; CUA atomic-event coalescence remains a possibility. CV-03 requires this coherent movement journey before closure, even though fuller transforms remain CC-04. The exact compiled source must be requalified after correction.

Independent report SHA256: `b2f8d8c8d73c639c77d0def018de4ef4cf9973759b96dd9ef02e6f6bc6e0b78a`. Its verdict is **changes required**, not native acceptance. Ordinary mid-drag Escape/focus, actual IME/touch, overlay interruption and zoom were not demonstrated. CUA only exposed atomic drag; scripted Escape and hosted ordered focus/touch regressions remain distinct evidence. Japanese paste survived in the accessibility value but the showcase default font displayed missing glyphs; multilingual rendering is unaccepted. The isolated review window was closed and root confirmed no showcase process remained; WriterCraft was untouched.

### Coalesced geometry input correction

Source inspection found that the existing geometry path initializes a move from the frame's final pointer position. A native batch containing press, movement and release therefore produces zero delta, or selects the destination instead of the pressed card. The connection controller already processes event positions in order. This is a concrete source defect consistent with the ordinary failure; the missing ordinary raw trace prevents attributing every failed drag to this cause before compiled follow-up.

The correction uses the actual primary press position for geometry arming and its viewport/layer/exclusion checks. Independent source review also found that the matching release position must terminate the gesture: later hover movement in the same batch must not move a clicked card or overshoot a completed drag. Current-frame host ownership and consumed connection input remain authoritative; interrupted input cannot rearm geometry or combine unrelated pointer cycles. Actual showcase-host regressions cover coalesced and separate-frame ordinary movement, exact Undo and embedded/interrupted input protection. Final independent source review, hosted execution and corrected native follow-up are pending.

The final independent source review found no remaining blocking findings in the movement correction and its four actual-host regressions. A separate bounded capture addition fills the zoom evidence gap: CUA exposes no modifier-scroll/pinch API, so the compiled fixture injects one Zoom event, verifies changed painted camera/port positions with unchanged canonical data/history, then runs every original connection step against fresh targets. All original27 steps and five readbacks remain; the two additional zoom steps make29. A fifth actual-host regression checks one injection and the next painted-frame boundary. The reviewer approved the combined source for hosted qualification. This is explicitly injected native zoom evidence, not OS pinch acceptance; corrected ordinary movement and overlay interaction remain required.

### Hosted actual-host checks expose retained input ownership defects

Source `e3f0f557088a9f61a794e2de0fbcabc7da938304`, [run42658967768](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42658967768), compiled and passed the existing model/adapter contracts, the new zoom check and the interrupted/outside-origin cases on all three platforms. Three actual-showcase tests failed consistently. Ordinary header movement before editing and exact Undo worked inside both movement tests, but movement after editing could not start because the TextEdit retained focus at the outside press. The embedded-origin safety assertion also caught a coalesced drag beginning inside the TextEdit and ending on the card margin incorrectly translating the card.

Keep these failing assertions intact. Correct the synthetic host's press-origin ownership and explicit outside-press focus handoff while preserving current text/IME input and draft content. This run is failed qualification and produced no accepted corrected native artifact; the authorized follow-up GUI window has not started. The next exact source must repeat hosted qualification before native review.

The follow-up changes only showcase host input handling: a guarded outside primary press relinquishes the exact previously focused TextEdit before the UI pass, while current text/key/clipboard/accessibility input, active/current composition and interrupted/foreign input keep their owner. A raw press inside the current clipped/enabled text response owns the entire frame even when the final pointer has left it; ordinary release ownership is also retained. No event is rewritten/replayed and no new focus is requested. The original three failing regressions remain unchanged. One actual-host regression adds pending text, preedit/commit and empty-preedit cancellation followed by a clean outside drag with preserved draft and one movement/history step. Independent read-only review approved hosted requalification; direct formatting/whitespace checks passed. Physical IME/native acceptance remains pending.
