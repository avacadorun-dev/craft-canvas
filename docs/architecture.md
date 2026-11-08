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
