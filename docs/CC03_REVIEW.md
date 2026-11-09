# CC-03 independent review

2026-11-09 source review, before hosted qualification. A separate read-only reviewer inspected the controller, geometry, model routing, view integration, targeted regressions and isolated showcase/capture infrastructure. The reviewer did not implement these files.

Initial findings were corrected: a named port at the legacy midpoint must not turn an unchanged reconnection into a mutation; coincident self-loop grips must expose both ends; a stationary pointer must yield to a newly appearing overlay; Space-drag must retain navigation precedence; touch-only input must not depend on synthesized mouse coordinates. Follow-up findings refined overlapping grip placement near adjacent nodes/clipping and Space precedence for an entire press/release batch. Focused tests retain these cases. Native observation logs now retain no-op/cancellation outcomes without requiring a canonical revision change.

The final source review reported **no known blocking source findings** and readiness for hosted qualification. Exact prepared-receipt/session/revision guards, target-start normalization, ordered release handling, bounded retained data and isolated artifact/source provenance were also inspected. Direct formatting, whitespace and Python syntax checks passed.

This is **source qualification only**. Rust compilation/tests, exact hosted build identity, actual compiled captures and independent four-journey interaction acceptance are pending. The issue remains open. A subsequent acceptance entry must identify the exact committed candidate and hash the reviewed evidence; this report does not claim physical touchscreen, downstream WriterCraft or production acceptance.

## Rework after the rejected native demo

The independent reviewer inspected the follow-up showcase source after CV-02/CV-03: actual scroll-widget ownership including release, collapsed failure-injection controls with immediate ordinary editing, visible blocked-state explanation and explicit host-reject/cancel before pending dismissal. The capture driver now publishes only enabled/unclipped action targets and retains the same27-step acceptance sequence. Upstream/scripted input tracing is bounded and redacts private payloads.

Six focused regressions exercise the actual host/header and trace redaction. Read-only review reported no blocking source findings and readiness for a new hosted candidate; formatting/whitespace checks passed. This does not convert the rejected native demo or failed capture into accepted evidence. The corrected exact build must pass hosted checks and a fresh independent native journey.

## Exact23b87d1 native review — changes required

The independent reviewer inspected the exact macOS artifact, all five actual WGPU readbacks and successful27-step scripted sidecar, then separately operated the ordinary unfiltered compiled showcase. [The progress record](CC03_PROGRESS.md#exact-native-capture-passed-ordinary-movement-remains-a-blocker) records the hosted run, artifact/sidecar hashes, observed interactions and evidence limits. Complete local review SHA256: `b2f8d8c8d73c639c77d0def018de4ef4cf9973759b96dd9ef02e6f6bc6e0b78a`.

Ordinary creation, either-end reconnection, target-start/same-node/parallel edges, Delete/Undo, strict invalid-target refusal, conflict recovery and embedded controls were observed. Repeated note-header and unobstructed body drags failed to move the card, including with no pending request or text focus. **P2 / acceptance blocker:** reproduce and correct the actual movement failure with a focused regression and exact compiled follow-up. The successful isolated fixture cannot override this CV-03 failure.

Ordinary evidence uses CUA screenshots/accessibility and visible source/revision/counts; no usable redirected event/canonical log exists for that process. Mid-drag interruption, physical IME/touch, overlay interruption and zoom remain unobserved through ordinary input. The pasted Japanese value was preserved but default showcase fonts showed missing glyphs. These limits are not accepted production exceptions. CC-03 stays open and no consumer adoption is authorized.

## Movement source correction review

Independent review found a further defect in the first draft: using the final frame pointer at release could move a clicked card after later hover, or overshoot a completed drag. The corrected private geometry input projection binds one accepted primary cycle to its actual press and matching release. It ignores trailing hover/independent cycles and conservatively aborts duplicate press, pre-terminal focus/pointer loss and Escape. Origin viewport/clip/layer/exclusion checks and existing host/connection ownership remain authoritative.

Four actual-showcase regressions cover separate/coalesced movement and exact Undo, movement after editing, outside/interrupted refusal, post-release hover/second-cycle isolation and embedded TextEdit-origin protection. Read-only re-review found no remaining blocking source finding in this bounded correction; direct rustfmt and whitespace checks pass. This is readiness for hosted execution, not corrected native acceptance or full CC-04 transform-order acceptance.

The reviewer also accepted the bounded zoom-evidence addition for hosted execution: one measured-center Zoom event plus an explicit painted-camera/changed-port check, then all27 original steps. The runner requires29 steps, unchanged canonical/history/counter gates, five real readbacks and retained zoom after the baseline. The actual Showcase regression verifies one injection and waits for the next painted camera frame. Provenance remains injected native events; no unsupported OS pinch is claimed. Corrected ordinary movement, moved-port interaction, Undo and applicable overlay checks remain the minimum fresh native follow-up.

## Host input correction after e3f0f55 qualification failure

The three hosted failures are preserved without weaker assertions. Independent re-review approved the bounded showcase fix: only the exact focused TextEdit may surrender focus on an eligible outside primary press, while pending text/key/clipboard/accessibility/IME and interrupted input retain their owner. Current clipped/layer-owned press geometry and drag-stop response retain text-origin ownership through release. The actual-host regression covers same-frame text/preedit/commit, empty-preedit cancellation with terminal ownership, then clean outside movement with the draft preserved. Deprecated egui0.36.2 Enabled/Disabled events are correctly untouched; cancellation uses the supported empty Preedit. No remaining blocking source finding was reported. Hosted requalification and exact native follow-up are required; physical IME remains unclaimed.

## d2bdaec scripted evidence accepted; ordinary gate incomplete

A fresh independent reviewer checked all five actual WGPU images, source/artifact/sidecar/image hashes and the read-only capture validator. The29-step injected-native journey passed, including zoom1.125, exact canonical restoration and expected Undo0/Redo2 history. Report SHA256: `fd66fe62ccd43d230a87b0484bb87e46d874c2e9e6f2651998337c5af09228cc`. This accepts the bounded scripted evidence only.

The ordinary attempt verified source but did not complete editing/movement. Its postmortem identifies an unrefreshed accessibility target after collapsing Testing and no focus observation before typing; unchanged text is therefore not proof of a product defect. No movement, moved-port/Undo or overlay evidence was obtained, and the owned process was stopped at the promised deadline. Postmortem SHA256: `32ffd1188882012407064dd2a30874d412cda57a1313de8efde1365da77aceaa`. Keep CC-03 open until a fresh bounded ordinary review completes with current native targets and explicit focus observations.

## Criterion audit after the successful owner scenario

The user confirmed all five ordinary actions on exact d2bdaec: text entry, header movement with draft retained, connection at the moved port, double Undo restoring the position while retaining text, and text selection without card movement. The independent reviewer accepted this as owner-reported native evidence for those actions and audited every issue/OpenSpec criterion against current hosted logs/capture plus the reviewed delta from the earlier23b87d1 ordinary observations. The controller, port geometry and model did not change; the changed movement/text paths have exact-current regression and owner evidence. Each hosted platform independently totals144 passing tests. Full criterion audit SHA256: `f0f72b6b5a31e5a7e25877ddfb1d26b022f30b7dd583342734b7c45d3981b4ac`.

One applicable gap remains: a compiled overlay must refuse a covered-port drag without graph movement or a retained preview, then disabling the overlay must restore creation and Undo while preserving the draft. Existing static-exclusion/stationary-takeover/terminal-consumption tests establish semantics but do not replace this named native observation. The reviewer confirmed that this representative refusal/recovery sequence is sufficient with the existing controls evidence; an additional inert-button click, exact counter reading or broad physical-input matrix is unnecessary. CC-03 stays open until that visible outcome is reported and reviewed. No unrelated feature or rebuild is required by this audit.

## Final disposition — accepted

The owner reported successful overlay refusal, recovery and Undo on exact d2bdaec, completing the final named native observation. The independent reviewer rechecked all scoped criteria and accepted CC-03 with no remaining applicable gap. [The complete disposition](CC03_ACCEPTANCE.md) records source, hosted checks, provenance, unchanged-code bridge and remaining later-scope obligations. Original report SHA256: `eeddf9223f91dc2bdd0d9fd674ae0a16b8f0cdd561252d0dd2fb674cd0f8cd1c`. This supersedes the preceding pending-overlay verdict without changing historical evidence. Root accepts the scoped milestone; the user's preview and downstream pins remain untouched.
