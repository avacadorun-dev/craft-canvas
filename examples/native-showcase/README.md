# Native CraftCanvas showcase

This unpublished example hosts the real `CanvasView` in eframe0.36.2/WGPU. Its three cards, legacy omitted-side edge and named ports are synthetic and held in memory. It does not open WriterCraft, load a project, register a document type or use the application's persistent settings. The window/app identity is `dev.craftcanvas.showcase`.

The current source is a CC-03 engineering candidate. Native gestures and production readiness require their own exact-build independent acceptance; adding this executable does not establish either.

## Interactive use

Run the exact hosted `craft-canvas-showcase` executable or isolated macOS bundle. Drag from a visible named or legacy port; two named source ports share the first card's right side. Select an edge to expose both reconnect handles. Its original edge remains canonical while dragging. Undo/Redo use the model's guarded history.

Ordinary mode applies edits immediately and exposes direct-manipulation help and Undo/Redo. **Testing controls** is collapsed by default. Open it to deliberately hold a prepared candidate: **Reject** retains the request, **Retry** revalidates it and **Apply held** acknowledges the exact candidate. **External revision** deliberately changes the synthetic canonical base so a pending request must expose conflict. These controls simulate a host boundary; none reports a durable disk save.

A pending/refused/conflicting request visibly explains why board editing is paused. **Discard pending** stays reachable outside Testing controls; the synthetic host cancels/rejects its held write before dismissing the presentation, without changing canonical data or history. A real asynchronous host must establish that cancellation/refusal itself before using the same recovery contract.

The note contains a native text draft, the scene body scrolls, and the third card contains a slider/button. The overlay checkbox covers one connection region with an input exclusion. These are deliberate embedded-input surfaces for independent review, not product features or alternate Canvas implementations.

Redirect stdout when collecting natural interaction evidence. Each canonical revision or pending-state transition emits an `observed-host-state` JSON line with the synthetic canonical data, history and lifecycle observations. Normal mode performs no in-app event injection; the provenance field distinguishes it from capture mode. These records still require the reviewer to identify the actual interaction method.

## Hosted capture

Build with `CRAFT_CANVAS_SOURCE_SHA` set to the exact source commit and the committed dependency lock. The coordinating workflow owns dependency resolution, build and artifact packaging. No local Cargo execution is part of this task.

```text
python3 scripts/capture-showcase.py \
  --executable /absolute/path/to/craft-canvas-showcase \
  --output /absolute/path/to/new-evidence-directory \
  --source-sha FULL_COMMIT_SHA
```

The runner launches and owns exactly one process, retains stdout/stderr and uses a bounded deadline. It never replaces a directory or terminates applications by name. The binary accepts `--capture-dir NEW_ABSOLUTE_DIRECTORY --scenario connections --source-sha FULL_COMMIT_SHA`; an explicit SHA must match its compiled identity when present.

The **Verification** window explicitly opens Testing controls for its 27-step scripted journey. It drives actual measured ports, edge handles and enabled, unclipped buttons with RawInput: create → hold → reject → retry/apply → reconnect target → apply → exact Undo twice → legacy omitted-side no-op → Escape cancellation. Five screenshots come from actual WGPU readback with matching request tokens and stable canonical/frame geometry. The sidecar records source identity, typed lifecycle observations, canonical data, history, measured targets and host submission/acknowledgement counts. A bounded input trace distinguishes unmodified upstream batches from scripted additions while redacting text, paste, IME, accessibility payloads and character keys. It does not identify an upstream event as physical input. PNG validation and file hashes are retained separately by the runner.

This evidence is explicitly **scripted native widget interaction**, not natural OS pointer/keyboard, physical touchscreen, full Canvas or production acceptance. Independent review must also exercise target-start/same-node/parallel/invalid creation, both endpoint revisions, native focus/outside interruption and embedded controls/overlay ownership on the exact hosted candidate. Keep that review separate from these bounded captures and preserve the user's running WriterCraft preview.
