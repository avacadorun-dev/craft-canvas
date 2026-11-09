# Upstream geometry provenance

The source application uses JSON Canvas1.0 and XYFlow connectors. The connector control-distance formula is derived from the MIT-licensed `@xyflow/system` package installed with the frozen source, `dist/esm/index.js` functions calculateControlOffset/getControlWithCurvature/getBezierPath. The generic implementation uses f64 typed geometry, bounded adaptive subdivision, explicit endpoint identity and host policy; it does not import the JS graph model or runtime. Retain upstream attribution for this port:

Copyright (c)2024 xyflow team. MIT License; see the source package LICENSE below. Standard JSON Canvas optional endpoint defaults remain no-arrow source and arrow destination. Explicit sides are preserved exactly; omitted handle positions are a deterministic generic presentation fallback, never a canonical rewrite.

Freehand candidate review is separate; no freehand or layout dependency is added by this connector checkpoint.

## WriterCraft #17 smooth-step correction

The original board's `CanvasConnection` calls `getSmoothStepPath`, not the previously ported Bézier helper. `SmoothStepConnector` ports the installed MIT XYFlow `getPoints`/`getBend` default routing (offset20, step0.5, radius5) into bounded f64 vertices and exact quadratic bends. The native adapter flattens those bends once for both paint and hit testing and returns the same derived label anchor to the host. No JS dependency or runtime is added. Standard arrow-end defaults are rendered; source-specific color/type catalogs remain host work.

Original loose-mode `CardHandles` declares Top first. When stored sides are omitted the native adapter therefore uses Top for presentation at both ends, retaining the omission in canonical data. Same-card opposite-side smooth-step routing in the source can cross the card body and become hidden/unselectable. This source defect is corrected with a bounded outside-card ring: shortest side route, clockwise opposite-side ties, and a complete loop for equal/omitted legacy handles. This derived correction changes no node geometry, edge fields or history.

Inspected frozen sources and SHA-256:

- `src/widgets/canvas-board/ui/canvas-board.tsx` (`CanvasConnection`, `CardHandles`): `be67e4c9d97a490df4053844e1cc317ce77d7098c8383299f5a3940300940d87`.
- Installed `@xyflow/system`0.0.79 `dist/esm/index.js` (`getPoints`, `getBend`, `getSmoothStepPath`, `getEdgePosition`): `42cc6461712c2b302bbc2b1890593af31415bc3a3e93d548c76f3d2b3e7268ea`.
- Installed package LICENSE: `023119ac20fb1c8c9930abe0bcd196989a1960388529a96fc43cebf96f07c9ff`. The existing XYFlow MIT attribution in `LICENSES/XYFlow-MIT.txt` also covers this port.

Hosted engine qualification and exact application native acceptance remain required before adoption/completion. This derived geometry does not accept full B15 endpoint authoring or B13/B16 style/schema workflows.
