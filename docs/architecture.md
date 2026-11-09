# Architecture decisions

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

Inline activation uses `begin_inline(ctx)` during the actual create/open UI frame, before the first mounted input batch. It binds the viewport and acquires focus only when the editor does not already own it. Repeated same-ID focus requests are forbidden during ordinary mounting/acknowledgement because egui0.36.2 turns them into native IME interruption. An immediate mount may recover focus from its creating pointer event only when no widget owns focus and no composition is active; it does not override another focused widget.

The host calls `prepare_inline_input(ctx, raw_input, blocked)` from `eframe::App::raw_input_hook`, only for its active retained editor. The hook first checks the bound raw viewport, native focus, host block/modal state and exact current widget focus. It never reads `Context::viewport_id` outside a UI pass. Unknown ownership leaves events untouched. Unmodified pressed Escape is one owned command: during current/frame IME it cannot cancel the card; otherwise a non-repeat queues one Cancel intent for the next enabled, focused mount. A valid non-composing Cancel mounts the native input read-only immediately, so text/paste in the same batch cannot mutate the retained draft or emit Done; keyboard ownership remains captured for that cancellation frame. Current-frame IME still takes precedence and does not freeze composition. The intent survives repeated hooks while eframe defers UI mounting; blocked/foreign input or a disabled/unfocused mount expires it, so a modal cannot leave a delayed Cancel behind. Text, paste and IME events are not rewritten or reinjected. This supports the existing single native workspace; it is not a new multiwindow router.

TextEdit captures Escape/arrows while leaving Tab for navigation. On the first mounted frame, before egui has installed that filter, an exact owned unmodified-arrow batch may schedule a global focus direction. After TextEdit, the adapter clears that direction only when the same widget still owns focus and the batch contains no Tab, accessibility, pointer-button/touch, focus, IME or unknown competing event. Hosts must not inject an unrelated programmatic focus-direction action into that same owned batch. The adapter does not unconditionally clear focus direction or mask platform IME output. Regression tests retain FullOutput and model actual creation handoff, prior IME-enabled input, first composition, acknowledgement, Commit, Cancel and ordinary neighbor navigation. Disabled retained editors use `interactive(false)` and do not process input, mutate composition ownership or request focus; the draft remains visible for host preservation/retry.

A host preserving an unacknowledged new-card draft can reparse `pending_node(session).raw_json()` with the current session limits, then prepare one exact-base `CanvasTransaction` containing `InsertNode(original_record, original_index)` followed by `EditText(expected_original_body, latest_draft)`. Sequential candidate validation preserves opaque record fields and numeric lexemes. This candidate is for an explicit host preservation path; it neither changes the canonical session nor satisfies the byte-identical original insertion acknowledgement. The ordinary `submit()` path continues to refuse an absent canonical card.
