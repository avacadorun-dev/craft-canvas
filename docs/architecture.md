# Architecture decisions

## ADR 008 — direct gestures over shared frame geometry

Selected 2026-11-09 for CC-03; qualification pending. The native adapter derives one bounded frame of named and implicit legacy anchors after host body measurement, then uses that frame for edge routes, visible grips, hits and interaction observations. Paint order keeps canonical edges behind bodies. Camera navigation is applied for the next frame so input never queries a different transform from the one painted.

An opt-in controller owns one initiating mouse/touch contact through an explicit terminal observation. It emits one guarded transaction and retains pending presentation until the host binds and acknowledges the exact prepared candidate. This remains a presentation state, not another graph or persistence authority. Host body/overlay ownership takes precedence. Coincident endpoint grips are visibly separated with leader lines while retaining exact canonical anchors and bindings.

Use an isolated synthetic eframe showcase for compiled interaction acceptance; the library crates gain no window runtime dependency. Hosted builds resolve and qualify its exact dependency lock and retain a source-bound macOS artifact. Neither this demonstration nor its in-memory acknowledgement establishes WriterCraft adoption or durable storage. See the [CC-03 design](../openspec/changes/direct-connection-gestures/design.md).

## ADR 007 — compatible named-port metadata

Selected2026-11-09 for CC-02. Store optional version1 `craftCanvas` metadata in the existing raw node/edge envelopes. Node-local `(kind,id)` keys are independent of geometric sides; edge node references remain standard `fromNode`/`toNode`. Preserve unknown versions and malformed known envelopes without interpreting them as editable defaults. Legacy endpoints remain available without migrating stored data. The [exact codec/API contract](../openspec/changes/named-ports-validation/design.md) defines absence, strict/loose validation, actual initiation, dynamic removal and reconnect policy.

Preview and final preparation share pure validation. Preparation resolves the complete candidate while authorizing reconnect against the original edge policy. Preservation copies retain unresolved imported bindings; deliberately retargeted copies receive authoring validation. Derived port caches and bounded copy provenance do not create another authoritative document. This model decision does not establish CC-03 native gesture acceptance or authorize a consumer repin.

## ADR 006 — production parity on the preserved authority

Selected 2026-11-09; implementation acceptance pending. The production target is complete observable React Flow capability through native Rust, then full Studio Canvas integration. Retain ADR001–005's canonical model, exact guarded changes and f64 projection. Expand the API through source-backed native equivalents rather than substituting an automatically mutated graph or browser runtime. The [production plan](PRODUCTION_PLAN.md) owns milestone/evidence boundaries; [React Flow inventory](REACT_FLOW_PARITY.md) owns source coverage.

Use React Flow12.12.1/system0.0.84 at `1bc405cd113618c17fd181b931a9fe573bec9fee` as the initial fixed comparison. egui-xyflow and other Rust graphs are source references, not unchanged dependencies. Upstream schedule is not a delivery gate. Ported code keeps exact provenance and license notices; no new dependency or global egui upgrade is selected.

The next model design must separate stable port identity from geometric side and distinguish raw legacy endpoints from their rendered defaults. Multiple handles on one side, dynamic measurements and strict/loose rules cannot be encoded as four side values. Typed connection preview validation is separate from final exact-base validation. Existing opaque envelopes remain canonical; named ports/parent relationships/custom graph fields require a versioned compatible codec with absent-field defaults and refusal of unsupported mutations before any data change. Do not overwrite previously unknown extension values to install a schema.

The interaction layer emits start/update/end/cancel observations and a single completed command intent. Observations cannot grant save/Apply authority. Pending host acknowledgements freeze conflicting commands and preserve drafts; original edges remain canonical throughout reconnection. Raw expected endpoints and displayed fallback ports stay distinct so dropping onto the unchanged displayed anchor is a no-op. Escape, focus loss, pointer loss, outside cancellation, base/board change and competing native controls end capture explicitly, including the release/cancel frame.

Rendering, hit testing and native accessibility derive from the same frame's geometry. Camera/parent/local-origin transforms are resolved before input queries. Host hooks cover node/handle/edge/label bodies and input exclusions; embedded editor/IME/media ownership has priority. Presentation caches are bounded and disposable; they never become another authoritative graph. A managed convenience API, if supplied, must use the same session commands as the controlled host API.

Generic geometry/input/view policies must be configurable while preserving the existing Studio compatibility profile. Parent transforms, snapping, origin/extent rules, source measurement, custom routes and headless bounds must have explicit semantics rather than inheriting application-specific integer rounding or fixed camera limits accidentally. API shape and extension spelling remain subject to CC-02's compatibility review; the narrow four-midpoint connection proposal is not frozen public API.

Ownership distinctions include pointer, keyboard/IME and wheel/scroll/pinch/touch rather than treating two booleans as a permanent complete contract. Before-delete policy can veto, transform or defer a proposed deletion; every deferred decision is revalidated against its captured exact base. A terminal interaction observation occurs even when invalid/cancelled/no-op input creates no transaction. Actual touchscreen evidence is separate from trackpad gestures and synthetic events.

## ADR 001 — model and native adapter

Accepted 2026-11-08. Two crates share one canonical board: a Rust-only model and an egui 0.36.2 adapter. The adapter emits semantic intents against a private session identity. Host UI and headless commands prepare through the same model and host validation. No app framework, alternate graph model, window loop or media runtime is imported.

## ADR 002 — opaque preservation

Accepted 2026-11-08. Preserve the complete original JSON bytes for no-edit export. Keep raw record envelopes and raw unknown top-level values so editing a known field never reconstructs unrelated opaque data through floating point. Reject duplicate keys at every nesting level before interpretation. Known geometry uses checked finite f64 values; host dialects preserve application extensions and validate resource ownership.

## ADR 003 — guarded immutable changes

Accepted 2026-11-08. Session identity is generated in process and never deserialized. A prepared change owns exact base and result documents; only private constructors can produce one. Apply checks its base; Undo/Redo prepare compensating changes at the current identity. Durable host checkpoint/save acknowledgements remain a separate required layer. A library Apply is not a database acknowledgement.

## ADR 004 — camera and budgets

Accepted 2026-11-08. Subtract an f64 world camera origin before converting visible local geometry to egui f32. Paint text in screen space at the requested effective font size. Input, nesting, graph counts, command count, geometry and retained history are bounded. History reports retained encoded/source payload accounting rather than pretending allocator overhead is zero.

## Adapter checkpoint API

`CanvasView::show(ui, session, dialect, host, config)` paints one canonical session and emits `CanvasIntent::Transaction` or explicit host actions. The host prepares through `CanvasSession::prepare`, validates project resources, checkpoints durably, then calls `apply`. `CanvasViewHost::paint_body` receives an intersected native clip rectangle and returns pointer/keyboard ownership; interactive media cannot silently become a drag. `TextCardEditor` captures exact base/old source and mounts inside the host's existing modal grid; it retains a draft on save failure and emits no mutation on Cancel. It does not pretend that its local acceptance is durable persistence.

The source editor's basic Save/Cancel component is intentionally separate from the board controller. The host blocks board shortcuts while it owns a dialog/editor. Native Canvas copy/cut/paste are intents, not ambient clipboard reads; app authorization and one captured raw bundle remain host concerns.

## ADR005 — history owns raw snapshots, not parsed historical boards

Accepted2026-11-08. Preserve exact immutable raw snapshots in past/future and parse only the chosen Undo/Redo target during preparation. The current source remains separately bounded12MiB and historical source32MiB, for at most44MiB retained encoded source under defaults. This fixes the original foundation's overly conservative charging of parsed historical caches, which could refuse an otherwise valid source Undo slot. Current parsed payload and stable-mapping metadata are measured/reported separately; allocator/container overhead and transient preparation memory are not falsely called zero. Step/node/edge/ID/input limits independently bound those structures. Preparation retains a complete candidate and exact base before host persistence; Apply remains a guarded acknowledgement.


## Pending text insertion input

`TextCardEditor::pending_insertion` owns one bounded standalone text record and the ordinary native draft. `CanvasViewHost::pending_text_editor` exposes only that presentation; the view renders it against the exact unchanged canonical session with board interactions frozen. No shadow session or copied graph becomes authoritative. `acknowledge_insertion` requires the same private session, exactly the next revision and byte-identical inserted record. It changes the binding alone, retaining native input identity, later text, selection, composition and Undo. Failed/stale/foreign acknowledgements leave the draft intact; host recovery and persistence remain host-owned. Generic event tests do not establish application/native OS acceptance.

Inline activation uses `begin_inline(ctx)` during the actual create/open UI frame. It binds the viewport and arms a transfer from that frame's exact focused widget; it does not focus an input that has not joined the completed accessibility tree. The next matching, native-focused, unblocked pre-input hook grants the transfer before egui handles the first text/paste/IME batch. Competing focus cancels the transfer. Mounting and insertion acknowledgement do not repeat the focus request. There is no insertion wait, event replay, platform-output masking, synthetic accessibility node or hidden warmup input.

The host must reveal and mount the intended card, or block the hook while the card is inactive, culled or disabled. A pre-input grant is subject to egui's normal unmounted-widget cleanup before AccessKit output; this prevents dangling focus but does not certify that an omitted editor accepted text. Native acceptance must verify actual mounting and exact retained text. Legacy immediate mounting may acquire focus where the real input joins the same pass's accessibility tree; its creating-pointer exemption applies only to that immediate path. A deferred activation's next pointer batch is a new interaction and can deliberately blur the editor. First-frame IME requires armed activation and the pre-input hook.

The host calls `prepare_inline_input(ctx, raw_input, blocked)` from `eframe::App::raw_input_hook`, only for its active retained editor. The hook first checks the bound raw viewport, native focus and host block/modal state, resolves any armed transfer, then requires exact current widget focus for Escape ownership. It never reads `Context::viewport_id` outside a UI pass. Unknown ownership leaves events untouched. Unmodified pressed Escape is one owned command: during current/frame IME it cannot cancel the card; otherwise a non-repeat queues one Cancel intent for the next enabled, focused mount. A valid non-composing Cancel mounts the native input read-only immediately, so text/paste in the same batch cannot mutate the retained draft or emit Done; keyboard ownership remains captured for that cancellation frame. Current-frame IME still takes precedence and does not freeze composition. The intent survives repeated hooks while eframe defers UI mounting; blocked/foreign input or a disabled/unfocused mount expires it, so a modal cannot leave a delayed Cancel behind. Text, paste and IME events are not rewritten or reinjected. This supports the existing single native workspace; it is not a new multiwindow router.

TextEdit captures Escape/arrows while leaving Tab for navigation. On the first mounted frame, before egui has installed that filter, an exact owned unmodified-arrow batch may schedule a global focus direction. After TextEdit, the adapter clears that direction only when the same widget still owns focus and the batch contains no Tab, accessibility, pointer-button/touch, focus, IME or unknown competing event. Hosts must not inject an unrelated programmatic focus-direction action into that same owned batch. The adapter does not unconditionally clear focus direction or mask platform IME output. Regression tests retain FullOutput, enable AccessKit, and feed real updates into accesskit_consumer. They cover late activation, an optional same-frame visible mount, prior IME-enabled input, first Unicode/paste/composition, acknowledgement, Commit, Done/Cancel, unmounted cleanup and ordinary neighbor/pointer navigation. Disabled retained editors use `interactive(false)` and do not process input, mutate composition ownership or request focus; the draft remains visible for host preservation/retry.

A host preserving an unacknowledged new-card draft can reparse `pending_node(session).raw_json()` with the current session limits, then prepare one exact-base `CanvasTransaction` containing `InsertNode(original_record, original_index)` followed by `EditText(expected_original_body, latest_draft)`. Sequential candidate validation preserves opaque record fields and numeric lexemes. This candidate is for an explicit host preservation path; it neither changes the canonical session nor satisfies the byte-identical original insertion acknowledgement. The ordinary `submit()` path continues to refuse an absent canonical card.


## Derived connector paths and hit testing

The #17 connector correction adds pure `SmoothStepConnector` geometry and additive `CanvasViewOutput.edges` frame evidence. Paint and pointer selection share the same bounded flattened rounded route; host labels use its source-compatible anchor instead of recomputing a midpoint. Arrow triangles follow stored decoded markers. Node hits and host input ownership take precedence. Edge-only Delete uses the existing exact-base RemoveSelection intent and changes no data before host acknowledgement.

Same-card paths use an outside-card ring to correct the original hidden opposite-side route, including a complete fallback loop for preserved equal/omitted handles. These are derived geometry decisions only. Source policies, field preservation, endpoint authoring, schemas/colors and durable authority remain unchanged. See `openspec/changes/visible-connector-interaction/` and `docs/upstream.md`; hosted qualification and native host acceptance remain pending.
