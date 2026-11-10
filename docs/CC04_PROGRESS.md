# CC-04 object manipulation progress

Updated2026-11-10. Issue [#4](https://github.com/avacadorun-dev/craft-canvas/issues/4) is the sole active library issue. CC-03 was accepted atd2bdaec and closed after the owner confirmed ordinary manipulation and overlay recovery; evidence commit5442038. No app pin or user preview was changed.

Reconciliation reuses the existing lossless model/commands/history, selection/movement/single-corner resize and spatial-group/frame distinctions. Remaining scope includes complete selection/lifecycle, sizing/aspect/rotation, persistent parents, locks/layering and host drop placement. Source inspection found a bounded first defect: merged host ownership/current exclusions gate geometry processing without always ending capture, so a blocked release can leave an active gesture. Existing legacy body capture already cancels; do not duplicate or regress that behavior.

The [OpenSpec change](../openspec/changes/object-manipulation/design.md) defines this first corrective increment and retains the full subsequent issue scope. The first private corrective diff is independently reviewed and hosted-qualified atfa4cf13; changed-source native acceptance and the complete issue remain pending. Root owns docs, integration and hosted dispatch; the bounded implementation and independent reviewer own only assigned files. Keep the owner's d2bdaec showcase and WriterCraft preview under their control.

The read-only reconciliation report has SHA256 `e841be5a9c3b36caadd86a11c025d4438d8a3da401db39ebac7930dc8dc28f47`. Additional gaps remain explicit: missing independent deletion policies (including incident edges), unconditional native rounding even when fractional model geometry is configured, and an old extreme-camera test that proves culling rather than a visible large-coordinate drag. Address them in their selection/geometry increments with focused regressions; they are not newly executed failures.

## First design accepted for implementation

Independent read-only review approved the private takeover correction before Rust edits. It requires accepted-cycle exclusion checks (including a stationary pointer), merged ownership cancellation and no same-frame rearming; a completed release must not be invalidated by later hover. Selection rollback/public lifecycle, sizing/parents and other policies remain following increments. This is design approval only. The implementation owner is limited to adapter view code and focused native-intent regressions; root retains documents, publication and hosted dispatch.

## Sizing contract retained for the following increment

A separate read-only original-source audit is retained in [CC04_SIZING_SOURCE_CONTRACT.md](CC04_SIZING_SOURCE_CONTRACT.md), SHA256 `218a32ea66c6085cd0604d29bb2ef9da2b55316cfd4edf513e4ecbd4bfdc1c51`. It distinguishes per-kind UI minima from storage limits, display size from original media bytes, shape-local rotation from whole-node transforms, and spatial groups from independent frames. Exact original dependency defaults and inconsistent source lock enforcement remain explicit design questions. This records requirements, not new implemented or native-accepted sizing behavior.

## First corrective diff ready for source review

Only adapter view code and three focused actual-egui regressions changed. Merged ownership terminates active Move/Resize/Marquee; current exclusions follow the accepted primary cycle, with retained last accepted position for stationary overlays. A matching release is insulated from later hover, and cancellation blocks same-frame geometry rearming. Pan, public connection output, schemas, dependencies and existing tests remain unchanged. Direct rustfmt and whitespace checks pass. Rust execution and changed-source native evidence remain pending; this is partial CC-04 work.

Independent source review found one collateral path in the first draft: permitting a valid geometry release despite later excluded hover also opened idle camera/other board actions under that hover. The correction must keep accepted-cycle geometry completion while independently retaining final-pointer exclusions for navigation, activation and keyboard dispatch. Extend the existing post-release regression with excluded wheel/zoom and unchanged camera; do not weaken the completed geometry assertion. This was found before publication or hosted qualification.

The final independent source re-review approved the corrected frozen diff for hosted qualification. Accepted-cycle geometry remains isolated from later hover; final-pointer exclusions independently gate camera, double-click and keyboard actions. The same regression now asserts wheel/zoom and Delete do not escape that boundary, the camera is unchanged, scrolling remains available to its owner, and the completed geometry has exact Undo. Active takeover requests another paint because bodies may already show the preceding preview. No remaining blocking source finding was reported. Exact-source Rust execution and eventual native issue acceptance are still pending.

## Exact first correction qualified

Source `fa4cf1369d20715db999921318d85dacb560e0ed` passed [run42669566429](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42669566429) on Linux/macOS/Windows:147 tests passed per platform, zero failures, formatting, the headless example, strict Clippy, unchanged committed dependency lock and native showcase build. The macOS ARM64 artifact was packaged by that run; it has not been launched or substituted for the owner's d2bdaec preview. Retained complete hosted log SHA256: `5317ae74f78fe4727851cd8db108fc285e57f2e16a6b2999f4827f6692faa1db`.

This qualifies the private ownership correction's source checks; it is not complete CC-04/native acceptance or an application handoff. No dependency/profile/capacity or public connection API changed. The next bounded implementation is the reviewed selection/lifecycle and geometry policy work, informed by the retained original-source sizing contract. Keep #4 active and preserve every remaining acceptance criterion.

## Original dependency defaults resolved

The separate [exact pinned dependency audit](CC04_PINNED_RESIZE_DEFAULTS.md) resolves ambiguity1 in the preceding source report. Official React12.11.2/system0.0.79 release archives match both registry and the original lockfile SHA-512 before extraction. The audit establishes eight controls, opposite anchoring, free aspect without a Shift toggle, top-left origin, grid-before-floored-delta behavior and size/extent constraints. It finds no explicit rollback guarantee to copy; the native contract retains explicit guarded cancellation. Original report SHA256: `3045fd4aa168e94c341df8d60ce71d72d4fd605b55993875a5cd91573793be99`. No installation/build or actual browser/native gesture was performed for this read-only research.

This evidence-only update changes no Rust source; the tested implementation identity remains fa4cf13. Further native/API work stays within CC-04, with full issue acceptance pending.

## Increment 2 — policy contract reviewed

Independent review approved [selection-policy.md](../openspec/changes/object-manipulation/selection-policy.md) before Rust edits. Defaulted node/edge policy hooks preserve all current defaults, including unknown-record selection/removal and existing NodePolicy literals. Removal must check both original and current records, including protected incident edges, before any candidate deletion. Native visibility/selectability use one derived rule; wrapper focus stays separate from embedded controls.

Review clarified two concrete cases: directly dragging a nonselectable node uses that node as the sole explicit root while preserving ordinary spatial expansion; and newly hidden targets must invalidate unsubmitted geometry/connection capture independently of selection. Pending connection preview can disappear while its exact request/receipt/acknowledgement stays retained. Empty initial selection is already reported, so the new change observation emits only differing filtered sets.

One implementation agent owns the model hooks/removal and focused tests; another owns adapter projection/input and its focused tests. Files are disjoint and both belong to #4. Root owns integration and hosted dispatch; the independent reviewer will inspect the frozen combined increment. Full geometry lifecycle, size/parent transforms and native complete-issue acceptance remain pending.

## Increment 2 — implementation ready for review

The resumed model/adapter work is integrated without resetting the retained draft. Model hooks and atomic removal checks have independent source approval. Four grouped model regressions cover original/current protection, incident/self-loop veto, separate accepted unlock, newly protected records, reconnect-away survivors, unknown/default compatibility and exact raw/history preservation.

The adapter adds shared derived visibility, per-frame filtered selection, independent nonselectable movement, wrapper focus, marquee inclusion and change-only selection snapshots. Unsubmitted captures cancel on relevant policy/visibility loss; hidden pending previews retain request/receipt/acknowledgement authority. Thirteen focused egui-event regressions cover these paths. A concrete old focus-ordering defect was also corrected: wrapper focus gain must not replace the current set before a Shift/Command pointer toggle.

Direct formatting and whitespace checks pass. The combined adapter diff is frozen for independent review; hosted execution and complete native acceptance remain pending. No dependency, stored schema, consumer pin or user preview changed. Crate guides distinguish the accepted d2bdaec source pin from these active development APIs.

The independent combined source review found no remaining blocking findings and approved exact hosted qualification. It checked visibility and hit consistency, direct drag roots/spatial membership, cancellation and retained pending authority, change observations and additive-focus ordering. This accepts source readiness only; the new regressions have not yet executed and no complete native CC-04 acceptance is claimed.

## First increment-2 qualification refused an outdated test API

Source `840764643f1966fdf0a8b7c4e88d8b80a7f8e72f`, [run42748945218](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42748945218), compiled the production crates but failed adapter-test compilation on macOS and Windows: egui0.36.2 RawInput has no modifiers field. No new regression execution or passing qualification is claimed. The bounded correction changes only the new event harness: deliver ModifiersChanged before the batch, including explicit NONE resets, instead of a removed RawInput field. All13 scenarios/assertions and production code remain unchanged. Independent re-review verified the pinned event/state API and approved exact requalification; direct formatting/whitespace checks pass.

Source `4ce017b45f854e91f31d536798ec6c8567caa6ba`, [run42749413664](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42749413664), compiled and passed all164 tests on macOS, including every new regression. Strict Clippy then refused two equivalent negated Option predicates in the new model test dialect. The focused correction uses is_none_or with the opposite comparison, preserving absent/true/other behavior; no production code, scenario or assertion changes. Independent review approved semantic equivalence; direct formatting/whitespace checks pass. The run remains failed qualification, and the next exact source must complete all checks.

## Increment 2 exact source and native compatibility smoke

Source `4ff564123a82483e6faf61ad12e0b77533b8c561` passed [run42749740099](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42749740099) on macOS/Linux/Windows:164 tests per platform, zero failures, formatting, headless example, strict Clippy, unchanged dependency lock and standalone native build. Complete hosted log SHA256: `f5623cd32c3c80f787230be5f5f53d0dda1883a27abfc25bba8a798f356cdcac`. The earlier4ce017b qualification failed Clippy on all three platforms; its passing regressions were retained without weaker assertions.

The downloaded macOS ARM64 artifact matched its source, committed lock, archive/executable hashes, architecture and ad-hoc signature before execution. Archive SHA256: `c802ff796e675e6d47200d511b4a508fd80a5586b18522f42348d8d11344daf3`; executable: `096acf8b53e235ab1907ad35775c78d88bf3397ee9462d8b1af8ee1ca265ca2b`.

A fresh isolated native run passed the existing29-step connection/zoom/acknowledgement/Undo journey in7.427 runner seconds (6.300916417 sidecar seconds), retaining five actual2400×1790 WGPU readbacks. Canonical bytes exactly restore the initial fixture, no pending request, Undo0/Redo2, four Begin events, two submissions, one no-op and one Escape cancellation. Expected view/host/acknowledgement counts remain unchanged, with no suppressed upstream input. Sidecar SHA256: `bdf72746c4c761583cf76fc247282d840e788e2fbc1ed1f636ca2c540207ee10`; runner report: `a2cf8283f26a8afda0b8a96b5d63d0ca8061ef4c0e771d48d8cd7c125a3aafea`. Root inspected the real connected-state image; source and painted route/ports match. The native process exited normally under its own runner; user-controlled previews were not operated.

This is current-source native **compatibility smoke**, with injected events and actual graphics readback. It does not exercise every new policy through ordinary OS input and cannot accept full CC-04. New policy semantics have exact-source egui/model regressions; full selection/geometry lifecycle, transformed stacking, size/aspect/rotation, nested parents/groups, host drops and complete independent native acceptance remain mandatory.

A separate reviewer independently recomputed source/artifact/sidecar/image hashes, inspected all five actual readbacks and reran only the read-only validator. The [bounded smoke disposition](CC04_POLICY_SMOKE_REVIEW.md) accepts29/29 compatibility evidence with no blocking finding; original report SHA256: `540da476fea430adeab33bab4c0b82dab09344f31a05330f2f9f9fbc4fa3cdc3`. It explicitly leaves new-policy ordinary input and full CC-04 native acceptance pending. This evidence-only update changes no Rust source; the tested identity remains4ff5641.
