# Architecture decisions

## ADR 001 — model and native adapter

Accepted 2026-11-08. Two crates share one canonical board: a Rust-only model and an egui 0.36.2 adapter. The adapter emits semantic intents against a private session identity. Host UI and headless commands prepare through the same model and host validation. No app framework, alternate graph model, window loop or media runtime is imported.

## ADR 002 — opaque preservation

Accepted 2026-11-08. Preserve the complete original JSON bytes for no-edit export. Keep raw record envelopes and raw unknown top-level values so editing a known field never reconstructs unrelated opaque data through floating point. Reject duplicate keys at every nesting level before interpretation. Known geometry uses checked finite f64 values; host dialects preserve application extensions and validate resource ownership.

## ADR 003 — guarded immutable changes

Accepted 2026-11-08. Session identity is generated in process and never deserialized. A prepared change owns exact base and result documents; only private constructors can produce one. Apply checks its base; Undo/Redo prepare compensating changes at the current identity. Durable host checkpoint/save acknowledgements remain a separate required layer. A library Apply is not a database acknowledgement.

## ADR 004 — camera and budgets

Accepted 2026-11-08. Subtract an f64 world camera origin before converting visible local geometry to egui f32. Paint text in screen space at the requested effective font size. Input, nesting, graph counts, command count, geometry and retained history are bounded. History reports retained encoded/source payload accounting rather than pretending allocator overhead is zero.
