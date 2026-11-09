# Stable named ports and compatible validation

Implement roadmap CC-02/issue2 after independently accepted CC-01. Existing endpoints identify a node and optional side, which cannot distinguish multiple handles on one side or validate dynamic ports. Add stable port identity, compatible extension semantics, pure preview validation and endpoint-specific reconnect policy while retaining legacy JSON Canvas behavior and exact guarded authority.

Reconcile all command paths before introducing new public mutation APIs. Source inspection found that `SetEdgeFields` can change node/side endpoints without entering the final `can_connect` validation used by `ReconnectEdge` and insertion. Correct that real bypass with a focused regression before port-aware validation extends the same boundary. Label/marker-only edits to preserved imported edges must not be newly rejected by endpoint eligibility checks.

No native gesture or new application pin belongs to this issue. Complete the model/codec API and hosted semantic checks before CC-03. Preserve opaque unknown versions and exact no-op bytes; typed mutation authority stays process-owned. Core extension schema/API design is recorded before implementation and independently reviewed.
