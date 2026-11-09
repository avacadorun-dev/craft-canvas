# One derived path for connector paint and input

`SmoothStepConnector` is a pure f64 route over exact side endpoints. It preserves XYFlow default routing and label locations, then flattens the rounded quadratic bends under `WorkControl` and an explicit complete segment limit. The adapter culls in world coordinates and only converts relative-origin visible paths to f32. Markers use the canonical decoded direction ends; omitted sides use the original first loose handle (Top) without writing a default into the record.

Same-card data uses an outside-card ring, including equal/omitted legacy handles. This deliberately fixes the source's hidden opposite-side line rather than treating imported loops as invalid. The ring is derived from node bounds and does not introduce a new document model.

The view paints and hit-tests the same rounded screen-space route, with node hits first and the last painted edge winning overlap. Edge selection is transient. Delete/Backspace emits the existing exact-base RemoveSelection command for node or edge selections; the host continues to own Prepare, durable save and Apply. Host pointer/keyboard capture, disabled/modal gates, focus and session cancellation remain authoritative.

`CanvasViewOutput.edges` is additive derived frame evidence: stable edge ID, path, marker triangles and label anchor. Hosts can position labels or contextual actions without recomputing a conflicting midpoint. This does not create an editing or persistence acknowledgement. Color/type semantics, authoring handles/reconnection and label dialogs remain host/application scope.
