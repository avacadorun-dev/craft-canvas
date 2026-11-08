# Upstream geometry provenance

The source application uses JSON Canvas1.0 and XYFlow connectors. The connector control-distance formula is derived from the MIT-licensed `@xyflow/system` package installed with the frozen source, `dist/esm/index.js` functions calculateControlOffset/getControlWithCurvature/getBezierPath. The generic implementation uses f64 typed geometry, bounded adaptive subdivision, explicit endpoint identity and host policy; it does not import the JS graph model or runtime. Retain upstream attribution for this port:

Copyright (c)2024 xyflow team. MIT License; see the source package LICENSE below. Standard JSON Canvas optional endpoint defaults remain no-arrow source and arrow destination. Explicit sides are preserved exactly; omitted handle positions are a deterministic generic presentation fallback, never a canonical rewrite.

Freehand candidate review is separate; no freehand or layout dependency is added by this connector checkpoint.
