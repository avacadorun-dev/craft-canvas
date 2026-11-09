# CraftCanvas product contract

## Current objective — production React Flow parity

The explicit 2026-11-09 priority is a standalone production-quality native Rust library with full functional React Flow parity, followed by complete original Writers Studio Canvas workflows through WriterCraft. The source-backed contract is [React Flow parity](REACT_FLOW_PARITY.md); the consumer boundary is [WriterCraft integration](WRITERCRAFT_CONSUMER.md); delivery and acceptance are defined in [the production plan](PRODUCTION_PLAN.md). These supersede treating the narrow foundation as the final library scope. Current implementation is partial; no production claim is made.

Preserve the existing implementation and authority rather than restarting. Use an exact upstream behavior baseline and explicit native equivalents. CraftCanvas owns delivery and fixes; waiting on another library's development is not a dependency. Unchanged third-party graph-library adoption is not selected. Stable named/dynamic handles, complete graph interactions, custom composition, viewport/query/event APIs, input/accessibility and measurable capacity must be covered before full parity acceptance.

## Preserved foundation requirements

CraftCanvas supplies reusable native board mechanics to hosts such as WriterCraft. The host retains business schemas, project authorization, asset capabilities, UI catalogs, durable revisions, recovery and assistant review.

The foundation must prove: parse synthetic complete JSON Canvas without losing no-edit bytes; preserve unknown envelope fields and large numeric lexemes through known edits; reject duplicate keys and invalid geometry before mutation; prepare/apply/undo against exact session identity; keep node/edge array order; cancel transient gestures on Escape, focus loss or base change; and retain precision at world coordinates ±10,000,000 and zoom 0.12–3.0.

Target source-compatible capacities are 12 MiB per snapshot, 5,000 nodes and 10,000 edges. Drawing and source-specific extension budgets remain host-dialect validation; no imported capacity may silently be lowered by the view. History accounts current/past/future payloads and has separate step and byte budgets.

Later reusable work includes drawing, shape/frame geometry, grouping/duplication, alignment/snapping, bounded layouts and detached export display lists. Its presence in this contract does not mean it is implemented or accepted. Source application templates, diagnostics, schema catalogs, timer/play-through and resource media remain host work.

No generic operation grants permission to fetch a URL, open a file, read a project or apply assistant changes. A deserialized request is input data only.
