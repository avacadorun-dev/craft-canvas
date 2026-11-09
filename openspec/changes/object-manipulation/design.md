# CC-04 staged design

## Existing authority and unresolved broader decisions

The accepted foundation already provides raw record preservation, exact-base commands/history, selection sets, spatial group expansion and distinct frame movement. Reuse it. Full node-origin/measured-size, persistent parent codec, resize/aspect/rotation, per-object selection/deletion/layering and host-drop APIs require their own reviewed detail before those increments start. The proposal retains them as mandatory; this first correction does not choose undocumented schema spellings or freeze a narrow final API.

## Increment 1 — active geometry takeover

The adapter clears legacy body pointer/keyboard capture, but merged CanvasInputOwnership and input_exclusions only gate the processing branch. An active Move/Resize/Marquee can therefore survive a blocked release and continue when the block clears. This is a source-inspected defect; reproduce it in the existing actual-egui event harness before claiming runtime verification.

Keep the existing private geometry state and exact transactions. Resolve the current accepted primary cycle against exclusions in event order, using the same press/matching-release projection qualified in CC-03. A current exclusion at any active pre-terminal position cancels; stationary exclusion appearance also cancels. After a matching release, unrelated hover/second-cycle positions cannot alter the completed cycle. Do not discard or rewrite native Text/Paste/IME events. Merged host ownership cancels existing geometry, consistently with legacy body ownership. Start gating and current layer/clip rules remain in force; this increment does not change outside-viewport drag policy, geometry rounding, pan gestures or public connection lifecycle.

Cancellation clears the disposable preview without canonical commands, stops same-frame geometry actions and cannot rearm from a release. Preserve current selection semantics; marquee selection restoration and the complete public geometry lifecycle will be specified in the following selection increment rather than silently invented here. Do not broaden the currently connection-specific output contract in this private corrective slice.

Owned code for this increment: crates/craft-canvas-egui/src/view.rs plus focused native-intent regressions. No model/schema, showcase/UI-label, lock or consumer changes. Root owns OpenSpec/progress and issue records. Independent review checks the bounded diff against the current connection/text regressions, then root publishes one exact hosted candidate. Fresh compiled full manipulation evidence remains required before issue closure; existing d2bdaec observations cannot verify changed code.

## Verification and later increments

Retain existing144 tests and add only regressions for the discovered takeover/release defect: move/resize refusal and clean recovery, representative explicit ownership channels, stationary/ordered exclusion plus post-release isolation. Assert original canonical bytes/history and absence of stale transactions. Hosted fmt/tests/strict Clippy run on Linux/macOS/Windows; no local Cargo. This does not claim touch/IME physical-device acceptance.

Following increments complete selection/observable lifecycle, compatible geometry and sizing policies, nested parents/group/frame transforms, layered hits and drop insertion. The detailed APIs and native journeys must be reviewed before implementation. The issue stays open through integrated native acceptance and the final complete-criterion audit.
