# CC-02 implementation evidence

Issue2 is complete at `99db5166f287831c96da7ba6630a2bb4d620ee4e`. The final acceptance below supersedes historical pending/failed checkpoints. No application dependency is changed.

## Generic field endpoint validation

The first bounded correction makes `SetEdgeFields` node/side changes enter the same final candidate eligibility check as explicit reconnection. Unrelated marker/label edits on preserved nonconnectable edges remain supported. Regressions cover atomic refusal, unchanged history, unknown numeric data and exact Undo.

Independent source review: PASS, no blocking findings, reviewer `cc01_final_review`. Reviewed `commands.rs` SHA-256 `4fc47150f97077b7954b7b9ab53fb28ab5d02c77cd0e9764c1bf530546873b49`; `tests/connectors.rs` SHA-256 `841257895be69cfb3f1d0d2b9efcb47bb765f2833288c6fe3cb509bc37938f64`. This review inspected final transaction ordering and compatibility; it did not run builds or establish native acceptance. Exact hosted qualification is pending for the ensuing commit.

Exact correction `f431e24782f4ca464a7516a711d3bd0974632e05` passed [hosted qualification42628265941](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42628265941): committed lock, Rust1.95 formatting, workspace tests and strict Clippy on macOS, Windows and Ubuntu. No native observation is applicable to this headless policy correction.

The public named-port contract and all other CC-02 acceptance criteria remain pending. Native drag interaction belongs to CC-03 and cannot be inferred from this model correction.

## Named-port candidate qualification

Candidate `02337e0ecf9eb9c05da2c603452dbb4259bea2d0` implements the reviewed codec, pure validation, guarded mutation paths and headless example. Independent source/API review is recorded in CC02_REVIEW.md. Hosted run42631366907 reached successful library compilation but refused the new `ports_codec` test crate because its crate-level documentation was missing. This is failed qualification, not passed tests. The subsequent correction adds that documentation only; all test assertions and production code are unchanged. The corrected exact candidate must pass the complete hosted workflow before issue closure.

Corrected test-crate source `e99b8d20fb21becdf2514755cd84b3d985c21ba5`, run42631778920, compiled and exposed a real existing parser gap: `node_only_reconnect_preserves_unchanged_legacy_side_lexeme` refused null before the reconnect began. The next focused correction decodes optional standard side strings as nullable, retaining the original raw representation and all wrong-type/unknown-side refusals. Independent review passed; test assertions are unchanged. Superseded runs can end cancelled because the workflow replaces their still-queued macOS job after a verified Ubuntu/Windows failure; this is never reported as a pass.

Source `1161731a345d4d63a9e2bda3b52d03ba5d05bfb8`, run42632319369, passed all104 workspace tests and the headless named-port example on Ubuntu/macOS/Windows. Strict Clippy then refused three `map_or(true, ...)` expressions and required equivalent `is_none_or(...)`. The next source changes those expressions only. Independent semantic-equivalence review passed; the entire workflow must still pass on that exact source.

Source `8be01193c6662696b16459957f68e9c28e61aa12`, run42632709158, retained passing runtime tests/example but then exposed the codec test crate's25 `expect` calls under the workspace lint policy. The correction uses fallible test results without removing assertions or adding suppressions. Independent review confirmed all8 tests and30 assertions remain. Qualification still depends on the corrected exact source.

## Final CC-02 acceptance

Exact source [`99db5166f287831c96da7ba6630a2bb4d620ee4e`](https://github.com/avacadorun-dev/craft-canvas/commit/99db5166f287831c96da7ba6630a2bb4d620ee4e) passed [qualification42633082977](https://github.com/avacadorun-dev/craft-canvas/actions/runs/42633082977): Rust1.95 committed-lock check, fmt, all104 workspace tests, executable headless named-port example and strict Clippy on macOS, Windows and Ubuntu. Cargo.lock is unchanged from6b68eb4; SHA-256 `6765794e3da4bb3323620d7b32fb3c7c9f31176222b865d48bbdff487015c68f`.

The four issue acceptance groups are covered: stable `(kind,id)`/same-side/fractional anchors and strict/loose/actual-start validation; optional v1 codec with legacy/unknown/default preservation; dynamic removal/final-state repair and original per-end reconnect policy; focused round-trip/Undo/stale/copy/raw-setter/bounds regressions plus independent public API review. The runnable example confirms saved endpoints and exact Undo of connection and port setup.

Independent review: PASS after the documented fixes, [CC02_REVIEW.md](CC02_REVIEW.md), SHA-256 `a068a4a6404579de0f8e0e8ba2990b9da21321374cb97e97e30d78b3a617df05`. That receipt includes final source supplements. Root accepts this complete model-scoped issue. Native observations are not applicable to this headless model/API issue; actual named-port painting, measured native handles and pointer creation/reconnection remain CC-03, not inferred from the example. Hosts retain storage/ID/policy authority; imported unresolved/unsupported data remains preserved without invented fallback. No production certification, registry publication or WriterCraft integration is claimed.
