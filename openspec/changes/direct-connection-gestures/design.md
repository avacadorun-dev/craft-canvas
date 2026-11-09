# CC-03 native connection design

Scope: handle drag creation, either-end reconnection, complete lifecycle/cancellation and exact acknowledgement. Reuse CC-02; keep one canonical CanvasSession. Native custom composition beyond these handles remains CC-05. Host storage, IDs, localization and resource authority remain outside the engine.

## Compatible public surface

Keep the two-field NodeBodyResponse and existing CanvasViewHost methods. Add default host methods:

- `allocate_edge_id(&mut self, base:ContentIdentity, origin:&PortEndpoint) -> Result<Option<String>>`: called once when an armed creation crosses its drag threshold. None declines creation; IDs must be nonempty, bounded and absent among current edges. No library random/global ID allocation.
- `input_ownership(&self) -> CanvasInputOwnership`: pointer, keyboard, ime, scroll and touch booleans, merged with legacy body flags. Hosts report current-frame ownership after bodies paint.
- `port_anchor(&self, node:&NodeRecord, key:&PortKey) -> Option<PortAnchor>`: optional current-frame node-local measured point/tangent. It refines geometry of an existing canonical named identity only; no new identity or canonical mutation.

CanvasViewConfig adds `connections: Option<ConnectionOptions>` (default None, preserving existing hosts), `input_ownership: CanvasInputOwnership` and `input_exclusions: Vec<Rect>` for overlays. ConnectionOptions defaults: drag threshold3points, target radius20points, painted port radius5points, reconnect radius8points, generation0. Reject nonfinite/out-of-bound options. Hosts increment generation when external policy or measurements change; a captured gesture cancels rather than using stale targets. Canonical data changes are guarded by ContentIdentity independently.

CanvasViewOutput adds current-frame ports, selected-edge reconnect handles, optional connection preview, ordered connection events and consumed_input. The frame's camera is reported explicitly. Defaults remain opt-in until a host supplies the connection ID/acknowledgement contract. The standalone showcase enables the feature and demonstrates the complete path.

## One bounded frame geometry

New port_geometry.rs derives canonical named and four implicit legacy anchors, keyed by node plus exact binding, with validated world position, side, screen center, clipped hit rectangle and availability. Named local coordinates can be fractional or outside the card; their culling is independent of body culling. Host measurements are queried after body painting. Data remains immutable; unknown/unresolved named metadata never falls back to a legacy anchor. Default legacy presentation for an omitted side remains Top and does not write that value.

Use the same geometry for port/edge paint, target selection, reconnect handles and observations. Exact painted hit takes precedence; otherwise nearest within radius, with deterministic opposite-kind/order tie handling. An invalid direct hit stays invalid instead of selecting another nearby valid port. Topmost visible nodes/handles and overlay exclusions take precedence over hidden geometry. Charge traversal/route work; keep count/ID/geometry budgets.

Resolve canonical edge endpoints through EdgeRecord::endpoint and the frame lookup. Extend SmoothStepConnector with loop_between(bounds, from, from_side, to, to_side) for actual named same-node anchors; legacy loop_around delegates without changing its result. Preserve distinct same-side anchors and parallel edges. Original canonical edges remain painted during reconnection and pending acknowledgement; the preview is additional disposable presentation.

Freeze one camera throughout paint/hit/intent computation. Idle scroll/pan may schedule next-frame camera and request repaint; never hit old geometry with a changed transform. Camera/viewport/board/lifecycle changes terminate captured connection state explicitly. Named geometry updates during capture require unchanged generation or cancellation.

## Controller and intent boundary

New connection.rs owns Armed, Dragging and Pending submission/rejection states. Store exact base, board widget ID, generation, initiating pointer/touch identity, origin, original edge/end/binding and displayed endpoint identity. The original document/edge never mutates while dragging. Threshold-only click does not create an edge. Target-start creation normalizes from/to roles while retaining actual start=To; strict/loose decisions use CC-02 validate_connection. Same-node distinct ports and parallel IDs remain allowed.

A selected edge exposes two larger endpoint handles; their hit priority precedes creation handles. Reconnect changes the chosen end while the opposite end initiates the candidate. Dropping onto the original displayed identity restores its original exact binding before comparison, so omitted/null/empty legacy sides remain no-op. A no-op emits no transaction and adds no history.

The original dragged handle's hit rectangle has priority when returning to it, including when a named port occupies the same legacy midpoint. Overlapping selected endpoint grips are visibly separated with leader lines; their true anchor and canonical identity do not move. Both ends of an equal-side self-loop remain independently reachable.

A valid release emits exactly one CanvasIntent::Transaction: ConnectPorts for creation or ReconnectPort for reconnection, at the captured base. Final prepare remains mandatory. State then becomes Pending, draws the submitted route and blocks conflicting board commands without repeating the intent. Invalid/empty release ends with an explicit outcome and no mutation. No preview success claims saved content.

Lifecycle observations are independent from commands: Begin, Update, End (submitted/no-op/invalid/empty), Cancel (typed reason). Every begun drag gets exactly one terminal observation. Armed clicks may terminate without Begin. Typed reasons include Escape, focus/pointer loss, outside, base/board/viewport/camera/generation change, disabled input and host takeover. Public cancel_transient/set_camera/set_selection/fit queue terminal observations rather than losing them. Terminal frames retain consumed ownership even after active state clears, preventing selection/move/double-click/edit shortcuts from reusing that release.

## Exact host acknowledgement and failure

Expose bind_connection_prepared(&PreparedChange): require pending base, a changed prepared candidate and matching final requested edge ID/endpoints. Store one bounded immutable exact candidate receipt, not another parsed graph or authority. The host binds after prepare and before persistence. acknowledge_connection(&CanvasSession) requires the same private session, exactly next revision and byte-identical bound candidate; then clear only pending presentation. This cannot report durable save independently of the host's checkpoint.

reject_connection(reason) retains the failed request/receipt for explicit retry or dismissal. retry_connection(session,dialect) revalidates unchanged base and policy and returns one new guarded transaction, then waits again. No automatic resubmission. Dismissal requires the host to have rejected/cancelled its submission; clearing presentation cannot revoke an in-flight persisted command. Unrelated base changes leave an explicit conflict with the retained request rather than silently dropping it. All retained source/IDs/error text are bounded.

## Ordered input arbitration

Bodies/overlays own input before connection handling. Pointer, keyboard/IME, scroll/pinch and touch ownership are explicit; neither cancelled ownership nor current keyboard capture may restart a pointer gesture in that frame. Captured connection frames suppress competing geometry/selection/text-open/clipboard/delete actions and report ownership to the outer host.

Process pointer/touch input in event order. A normal touch release followed by PointerGone in the same batch completes before Gone; Gone before release cancels. Capture one touch ID/device; another touch cannot complete it, and synthesized pointer events must not double-submit. Native focus loss, outside release, missing pointer without normal release, Escape, host ownership and changed exact context cancel. No physical touchscreen acceptance is inferred from injected events.

Missing mouse coordinates cancel only mouse capture; touch input uses its own ordered positions and terminal events. An overlay appearing under a stationary captured pointer also cancels. The existing Space+primary navigation chord precedes connection arming and remains available to the board pan controller.

## Standalone verification

Add publish=false examples/native-showcase crate with pinned eframe0.36.2 (wgpu/accesskit/default_fonts/x11/wayland), image0.25.10 PNG only and existing model/adapter. One synthetic in-memory graph includes same-side named ports, a legacy omitted-side edge with label/unknown fields, embedded text/scroll controls and a small delayed/rejected acknowledgement control. No WriterCraft project access or identity.

Resolve the changed lock on GitHub; review/commit it before qualification. Existing portable fmt/tests/strict Clippy remain required. Hosted builds retain an isolated macOS ARM64 engineering executable/bundle with exact source/compiler/lock/binary hashes. Capture actual WGPU readback plus bounded sidecars; scripted RawInput and natural OS interaction remain separately identified. Run the downloaded candidate in its own process/window; preserve the user's WriterCraft Preview.

Independent acceptance uses four journeys: build a connected graph (including target-start/same-node/parallel/invalid), revise both endpoints and exact no-op, interrupt/recover including delayed/refused acknowledgement, and operate embedded controls/overlays. Inspect compiled captures and interactions plus canonical/history outcomes. Source checks or screenshots alone cannot close CC-03.
