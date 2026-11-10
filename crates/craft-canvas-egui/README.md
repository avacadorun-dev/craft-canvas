# craft-canvas-egui

The native egui interface for [CraftCanvas](https://github.com/avacadorun-dev/craft-canvas). It projects a host-owned `CanvasSession`, paints native bodies and connectors, and emits typed requests. The application retains storage, domain rules, asset capabilities, clipboard access and window/event-loop ownership.

This crate is intended for standalone native graph/canvas applications. It has no WriterCraft, React, WebView or browser dependency. **Complete React Flow functional parity and production qualification are in progress.** The current viewport is an engineering API; it does not yet provide the full planned graph editor interaction surface.

## Installation

Current baseline: **Rust 1.95**, edition 2024, **egui 0.36.2**. The planned crates.io name is `craft-canvas-egui`; **it has not yet been published there**. Pin both packages to the same reviewed revision and use compatible egui types:

```toml
[dependencies]
craft-canvas-model = { git = "https://github.com/avacadorun-dev/craft-canvas", rev = "d2bdaeca02ca028693222cf16153ff7c34c7b874" }
craft-canvas-egui = { git = "https://github.com/avacadorun-dev/craft-canvas", rev = "d2bdaeca02ca028693222cf16153ff7c34c7b874" }
egui = "=0.36.2"
```

This is a source pin, not a production-readiness claim. egui types from another version cannot be passed directly to this adapter.

## Current surface

- Retained camera/selection, relative-origin screen conversion, clipping and observed node/edge geometry.
- Node/edge selection, movement, marquee, single-corner resize, pan/zoom/fit and board shortcut intents.
- Routed connectors, marker triangles, shared paint/hit paths and edge-only deletion intents.
- `CanvasViewHost::paint_body` for native bodies with pointer/keyboard ownership.
- `TextCardEditor` for retained exact-base drafts, in-card input, pending insertion and acknowledgement; explicit raw-input integration for IME/focus ownership.

The pinned d2bdaec revision additionally has scoped CC-03 acceptance for measured named/legacy ports, primary dragging and either-end reconnection with explicit lifecycle and exact host acknowledgement. Enable `CanvasViewConfig::connections` with `ConnectionOptions`, implement host edge-ID allocation, and bind/acknowledge each prepared connection through the view API. The [standalone showcase](../../examples/native-showcase/README.md) demonstrates that boundary. Complete custom graph UI, parent/subflow interactions, richer viewport controls, physical touch and full graph accessibility remain pending. Model support for a command does not prove a reachable native gesture. The [ordered roadmap](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/ROADMAP.md), [parity inventory](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/REACT_FLOW_PARITY.md) and [production plan](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/PRODUCTION_PLAN.md) track the complete goal.

## Mount a board

Retain `CanvasView` and `CanvasSession` outside the UI callback. The session is the canonical graph; the view contains disposable interaction state.

```rust
use craft_canvas_egui::{CanvasView, CanvasViewConfig, CanvasViewOutput, PlainTextBody};
use craft_canvas_model::{CanvasSession, StandardDialect};

fn show_board(
    ui: &mut egui::Ui,
    view: &mut CanvasView,
    session: &CanvasSession,
) -> CanvasViewOutput {
    let config = CanvasViewConfig::new(egui::Id::new("example-board"), ui);
    view.show(ui, session, &StandardDialect, &mut PlainTextBody, &config)
}
```

The function returns the complete frame output for host dispatch. `PlainTextBody` draws inert text; it is not a rich editor or media implementation. Give each board a stable distinct ID, supply localized `CanvasViewLabels`, and replace the body host as needed. The adapter uses the host's egui appearance rather than opening a separate application frame.

## Intents, callbacks and persistence

Handle `CanvasViewOutput.errors` actionably and dispatch its `intents`. `Transaction` carries a frozen canonical base. Prepare it through `CanvasSession`, validate/checkpoint the candidate against the host's durable revision, then call `apply` after success. Prepared Undo/Redo use that same path. Resource opening, clipboard, text creation and ID allocation are explicit host actions, not ambient side effects.

`CanvasViewHost::paint_body` receives reserved body geometry in a clipped native UI and returns `NodeBodyResponse`. Controls must report pointer/keyboard ownership so the board does not turn their commands into movement or deletion. Use `CanvasViewConfig::interactive` and `shortcuts_blocked` for host modal/pending states. Cancel transient view state when changing projects, views or input ownership. Preserve drafts on stale or failed acknowledgements.

For native text editing, retain one `TextCardEditor` rather than reconstructing it every frame. Mount it in the intended visible card, arm inline activation through its API, and call `prepare_inline_input` from the host's raw-input hook with the correct blocked/active state. Its insertion acknowledgement binds the original exact record; it is not permission to overwrite later text. The [architecture's input contract](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/architecture.md) documents focus transfer, composition, Escape and acknowledgement. The inert-body example above does not configure that editor lifecycle.

The current adapter has known lifecycle gaps addressed by production work, including explicit pointer-loss/outside-drop cancellation and richer scroll/gesture ownership. Do not infer complete gesture isolation or accessibility from body callbacks. The maintained contract will cover these interactions without introducing a second graph authority.

## Selection policy in the CC-04 development branch

The active CC-04 work adds dialect-selected visibility/selectability, independent board-wrapper focus, and `SelectionMode::Intersect`/`Contain` for marquee inclusion. Hidden nodes omit their bodies, handles and incident routes from native presentation while their canonical data remains intact. Visible nonselectable objects still block clicks through them; a movable nonselectable node can be dragged as its own explicit root, retaining ordinary spatial-group expansion.

`CanvasViewOutput::selection_change` reports the final filtered stable set only when it changes. Programmatic selection is filtered on the next frame; the initial empty set and unchanged frames emit no event. This observation grants no mutation authority. Full geometry/selection lifecycle remains subsequent CC-04 work.

New visibility cancels affected unsubmitted captures. A hidden pending connection suppresses its preview while retaining its exact request, receipt and acknowledgement; hiding is not cancellation of a submitted save. Wrapper focus flags never own an embedded TextEdit or media control. Forward Delete requests retain the full selected set, and the model can refuse protected deletion atomically.

These additions require the development revision in [CC-04 progress](../../docs/CC04_PROGRESS.md); the qualified pin above predates them. Native acceptance of the complete manipulation milestone remains pending.

## Platform and verification status

Hosted formatting, locked tests and strict Clippy run on Linux, macOS and Windows. Synthetic egui events and AccessKit-focused editor checks are engineering evidence; they do not replace real OS input, graphics or accessibility acceptance. macOS ARM64 is the first native product qualification target. Full standalone production acceptance and Windows/Linux native interaction remain pending.

Use focused regressions and compiled native review when changing interactions. Keep public examples synthetic and preserve notices. The current contributor workflow performs Cargo builds/tests through hosted CI; see the [verification policy](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/verification.md).

## License and provenance

MIT. The adapter uses CraftCanvas's attributed connector geometry; [upstream provenance](https://github.com/avacadorun-dev/craft-canvas/blob/feature/native-canvas-foundation/docs/upstream.md) records XYFlow source and notices. React Flow is a functional reference, not a runtime dependency or affiliation claim.
