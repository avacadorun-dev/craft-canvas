# WriterCraft board visual direction

Date: 2026-11-09. Status: **independent designer proposal; owner review pending**.

The previous designer references covered the editor. They did not establish the
appearance of a board inside WriterCraft. This is the first explicit board
composition proposal. Neither the approved editor direction nor a running
CraftCanvas engineering demo constitutes acceptance of this board design.

## Recommendation

Keep the existing WriterCraft window and project navigator. Give the board one
quiet identity row, readable content cards, thin neutral relations, and blue
selection. Put editing and styling beside the selected object. Make connection
handles visible and give both edge endpoints direct manipulation. Writing remains
inside its card with unambiguous input ownership.

The proposed composition uses a synthetic project, four cards and three labeled
relations. A document card is distinguished from an independent note by its icon,
type label and document action. The note about Anna is not a new canonical
character entity. The layout does not prescribe manuscript order, synchronization,
promotion, copying or deletion semantics.

## Reference provenance

| Reference | What informs this proposal | Boundary |
| --- | --- | --- |
| WriterCraft `docs/design/DESKTOP_UX.md`, `WORKSPACE_EDITOR_REDESIGN.md`, and `assets/workspace-balanced.svg` | Existing Balanced window proportions, navigation, typography, restrained surfaces and shared chrome | These are editor references, not earlier board approval. The old mascot in historical artwork is not reused; current branding remains the approved lion. |
| [React Flow custom nodes](https://reactflow.dev/examples/nodes/custom-node) | Content-specific node surfaces, readable hierarchy and distinct handles | Interaction/composition reference; no React runtime or web UI is proposed for WriterCraft. |
| [React Flow handles](https://reactflow.dev/learn/customization/handles) | Spatially identifiable endpoints, multiple handles and explicit valid-target feedback | App validation and stable IDs remain governed by the existing host contract. |
| [React Flow reconnect edge](https://reactflow.dev/examples/edges/reconnect-edge) | Direct manipulation of either endpoint | An accepted reconnect still requires the engine/host transaction and cancellation rules. |
| [React Flow edge toolbar](https://reactflow.dev/examples/edges/edge-toolbar) | Compact contextual controls readable independently of graph zoom | No permanent toolbar containing every action. |
| [egui-xyflow at 1011740](https://github.com/avinkrisv/egui-xyflow/tree/1011740bb6f417dfffa10a11dcaf1e22a8ba1458) and its [upstream Basic Flow image](https://github.com/avinkrisv/egui-xyflow/blob/1011740bb6f417dfffa10a11dcaf1e22a8ba1458/screenshots/basic_flow.png) | The user-supplied native Rust reference and its graph-oriented presentation | MIT upstream. Its source/image provenance was verified; the external image did not load in this review environment, so no claim of pixel-level visual inspection is made. No claim that upstream is complete or abandoned. |

The board concept is a designer-authored adaptation, not a screenshot copied from
these sources and not a screenshot of the application. The proposal does not
depend on another project's development schedule.

## Composition and metrics

All measurements below are proposed logical points at 100% zoom. Preserve native
platform scale and font rasterization when translating them to the native egui host.

| Element | Proposed rule |
| --- | --- |
| Window | One native window. Keep the real macOS system controls; the concept's grey circles only reserve their location. No nested app frame. |
| Global bar | Existing 48-point shared bar and project context. |
| Project navigator | Existing Balanced default 264-point width; preserve its current resizing and collapse behavior. This proposal does not redesign the tree. |
| Board identity | 56-point row: board title, one Add entry, fit and overflow. Do not carry the editor's formatting bar onto the idle board. |
| Background | Quiet graphite/off-white surface, subtle 24-point dot interval at the illustrated zoom. The grid is subordinate to content. |
| Card | Opaque surface, 8-point corners, 16-point internal padding. Reference content width approximately 280–320 points; preserve existing object sizing behavior. |
| Typography | Source Sans 3 / native fallback. Card title 17, body 15 with 1.55 line spacing, type/relationship labels 12. Existing app font preferences remain authoritative. |
| Card hierarchy | Small type icon and label, title, readable body. Document cards have a thin accent strip and a labeled document action; independent notes do not imply live document synchronization. |
| Relations | 1.8-point neutral idle stroke, 2.3-point active stroke; rounded changes of direction and small open arrowhead. Labels get an opaque board-colored backing and modest padding. |
| Handles | 8-point hollow idle mark, 10-point selected/active mark; effective hit region at least 20–24 screen points at supported desktop zoom. Do not shrink the hit region with graph scale. |
| Context actions | A compact surface beside the selected object; visible Edit and a few relevant actions, overflow for secondary choices. Keep screen-size control labels stable at zoom. |
| Selection | Restrained blue border/ring. Valid target adds a distinct halo at the actual receiving handle; an invalid target adds a dashed outline and text reason. Never depend on color alone. |
| Navigation aids | Small footer/zoom cluster. Fit remains reachable in the board identity row. The concept does not introduce a new permanent inspector. |

Dark direction follows the existing graphite family: background `#191c21`, panel
`#21252b`, board `#1c1f24`, text `#e8ecf3`, accent `#6da4ff`. Proposed card surface
`#282e37` and relation `#8b97aa` separate cards/edges without bright outlines.
Light direction uses white cards on a cool pale board with accent `#356fd0`.
These are presentation tokens, not new palette choices. Appearance and palette
remain independent; legacy palettes must retain readable derived states.

Use the existing coherent Lucide icon family with labels or tooltips. No emoji
icons. The concept uses the wordmark only; do not revive the historical monk.

## Component and state inventory

| State/component | Visual contract | Concept coverage |
| --- | --- | --- |
| Default board | Content and three canonical relations are readable; there is no permanent selection toolbar or instructional overlay. | Shown. |
| Hover/focus | Show a subtle handle/edge affordance without repainting every card. Preserve visible keyboard focus and accessible action names. | Required for native implementation; hover/focus is not simulated exhaustively. |
| Selected card | Blue outline, visible connection points, compact adjacent Edit action. Double-click existing editable content reaches its editor. | Shown for an independent note. |
| Selected edge | Blue relation and distinct grips at both ends. Selection must not require aiming at a one-pixel path. | Shown as a state; no actual dragging is implemented in the concept. |
| New connection | Separate dashed draft path, source handle and valid target feedback. Existing canonical edges remain visible and unchanged. | Shown from the first note to the letter note. |
| Invalid target | Dashed invalid target, invalid preview and an actionable host-supplied reason. No canonical mutation. | Illustrative refusal state; the concept invents no new validation rule. |
| Reconnect | Drag either endpoint; keep the original relation until acceptance. Escape/rejection restores the exact original endpoints and IDs. | Endpoint affordances shown; transactional behavior must be verified natively. |
| Inline writing | Actual text editor inside the selected card. Text caret/selection, IME and shortcuts own input; typing cannot drag nodes or trigger board Delete. | Editable local concept field shown. No native input or persistence claim. |
| Pending/rejected edit | Distinct saving/pending state and actionable failure; preserve exact draft on rejection. Never show Saved before host acknowledgement. A held/conflicted request must expose a clearly labeled recovery or discard action and explain what remains unchanged; it cannot leave the whole board apparently inert. | Required native state; not fabricated in the concept. |
| Document card | Clearly identifies the referenced document and its existing open action. Return navigation preserves board context. | Shown visually; navigation is not connected. |
| Shape/Mermaid/media | Editing remains directly discoverable, including existing-object double-click. Images expose resize affordances while preserving original media. | Required by existing roadmap and consumer contracts; not illustrated in this four-card slice. |
| Empty board | One clear entry to existing creation actions, no sample content presented as the user's work. | Not illustrated. |

The preview state controls sit outside the depicted product. They are review
controls, not proposed product tabs. Optional spacing/stroke adjustments are
design exploration, not new application settings. The narrow conversation layout
stacks cards and reroutes the same three relationships solely to keep the reference
readable. It is not a proposed automatic rearrangement of a native user's board.

## Ownership and implementation boundary

**Library theme/mechanics:** geometry, edge paths/labels, handle and endpoint hit
regions, hover/selection feedback, connection/reconnection draft state, screen-size
controls, input ownership and drawing hooks. Keep defaults generic and host theme
inputs explicit; WriterCraft domain labels do not belong in the engine.

**WriterCraft composition:** the shared window, project navigator, board identity,
localized commands, note/document/media content, existing domain references,
save acknowledgement, history, compatibility and return navigation. A separately
owned application workstream adopts an exact qualified library revision.

CC-03 remains the active engine issue. This design does not start another product
implementation issue, approve a dependency repin, or certify its current candidate.
Later direct editing/resizing and workspace work stay in their roadmap order.
All applicable B01–B26 scope remains intact; a small showcase is not their coverage.

## Native designer acceptance

For each implementing slice, review the exact committed engine/app revisions and
actual compiled native captures. Use bounded existing workflows, not a new
exhaustive screenshot matrix:

1. In the real WriterCraft shell, inspect default/selected content at the normal
   desktop size and minimum supported window size in light and dark appearance.
   Check readable body text, contrast, relation labels, no clipped menus, and no
   accidental second toolbar or frame.
2. Create one connection by dragging, cancel it, reject an invalid target, then
   accept a valid target. Check preview/canonical distinction and visible target
   feedback. Reconnect each endpoint and cancel once. Verify at ordinary and
   zoomed-out scales that handles/edge hit regions remain usable.
   Hold or conflict one request and verify a clear pending/error explanation,
   obvious recovery, and the absence of an unexplained board-wide input lock.
   A first-time observer must be able to discover direct card/handle manipulation
   without believing that all graph interaction requires command buttons.
3. Select a card and reach editing by double-click and the visible contextual
   action. Enter a real native draft, select text and use its editing shortcuts.
   Check that board drag/delete does not steal input and a rejected save preserves
   the exact draft. Use the existing shape/Mermaid/image scenarios when their
   roadmap slice is under review.
4. Open a referenced document and return to the same board context. Confirm that
   layout, zoom, text editing and cancellation have not altered manuscript order,
   document semantics or original media.

Blocking findings require rework. Record exact source, native artifact/capture
hashes, named interactions, reviewer outcome and accepted limitations. Owner
feedback on this first proposal must update implementation/reviewer guidance
before native acceptance. Neither this file nor the interactive concept is an
accepted native UX gate or evidence of full React Flow/WriterCraft parity.

## Proposal review evidence

The conversation concept is `canvas-design-direction.html` in the thread's
authorized visualization directory. It was rendered and visually inspected in an
isolated headless Chrome profile on 2026-11-09, without accessing the running
WriterCraft preview or an existing browser profile.

Observed: dark desktop connection state, light desktop overview, and 320-point
conversation reflow were readable without horizontal overflow. Overview, selected
edge, connecting, invalid and inline-writing modes executed without JavaScript
errors. Canonical counts remained four cards and three edges in draft/refusal
states. The narrow presentation preserved the same edge endpoints. The upstream
image was unavailable and its embed was removed; primary links remain.

This verifies only the reference presentation. Native rendering, real drag
gestures, accessibility, localization catalogs, persistence and performance remain
unverified by this concept. No Rust code, application behavior, engine dependency,
build, release or existing preview was changed to produce it.
