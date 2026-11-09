# WriterCraft board visual direction

Date: 2026-11-09. Status: **independent designer proposal; owner review pending**.

The four-card v1 composition below is historical. The later full-board breadth
correction supersedes it as the representative design reference. See
[Revision v2](#revision-v2--mixed-content-boards) for the current proposal, complete
capability placement map and remaining design work. Neither revision is compiled
UI or accepted native functionality.

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

## Revision v2 — mixed-content boards

The three private original-Studio screenshots supplied later on 2026-11-09 show a
substantially broader workspace than the v1 graph: multiple named frames, nested
groups, typed/status cards, labeled relations, drawn objects and shapes, images,
PDF, audio/video controls, URL and other-board cards, tool groups and a minimap.
These screenshots were inspected locally; neither the screenshots nor their
project text/media were copied into the public repository or the concept. Visible
objects establish presentation evidence, not every creation/edit/save operation.
The source-backed B01–B26 consumer contract remains the functional authority.

**Correction:** a visually restrained board must retain this breadth. A cleaner
toolbar cannot mean fewer object families or missing professional workflows.
The previous four-card composition is insufficient as a reference for the
complete board. It remains useful only for its explicitly bounded interaction
states. This correction does not start another active implementation issue;
CC-03 continues to own direct connection input and recovery.

### Current composition

`canvas-design-direction-v2.html` is a separate, synthetic concept, preserving v1.
It depicts the same native WriterCraft shell with two board states:

- **Research and media:** nine content objects in two named frames: an original
  synthetic station illustration, two notes, PDF, audio, video, URL, other-board
  reference and unavailable external-file fallback. Rectangle, ellipse and pen
  stroke objects remain visible alongside three labeled relations.
- **Story and connections:** eight content objects across two named frames,
  including document references, independent notes, dialogue/state labels and an
  other-board reference. A nested group contains two alternatives. Five labeled
  relations, a diamond, rectangle, ellipse and drawn arrow show mixed composition.

The content and vector art are newly authored fictional examples. Media transport
is illustrative; no audio/video playback, PDF decoding or filesystem/network
operation occurs in the reference. The external-file card demonstrates an inert,
unresolved reference and an actionable host recovery placement, not automatic
access to a path.

The model of access is deliberately small:

1. The shared application chrome retains project context, search and assistant.
2. The board row holds identity, view selection, Add and board actions. Add contains
   existing content families and template access, rather than another permanent
   row of buttons.
3. A six-entry tool rail holds selection, pan, drawing tools, shapes, frame and
   objects/layers. Drawing reveals pen/highlighter/eraser; shapes reveal the five
   original shape families. These are groups, not removal of their alternatives.
4. Selection reveals a compact context row with Edit, Properties and object
   actions. Properties is disclosed only on request. Copy/Cut/Duplicate,
   alignment, color, layering, locking, promotion and Delete retain contextual
   placement and keyboard equivalents. Commands must be enabled only for their
   valid object kind/selection in the real host.
5. A quiet minimap and zoom/fit cluster retain spatial navigation. These may
   collapse in a small real window only with a visible recovery command.

The v2 concept currently collapses the project sidebar below its review breakpoint
and stacks content at narrow conversation widths. This is **reference reflow only**,
not permission to rearrange a native canonical board when a window is resized.
Real camera/selection and document order remain unchanged by presentation.

### B01–B26 placement and outstanding depth

“Shown” below means a design reference, not implementation or acceptance. A named
menu location alone does not complete its dialog or workflow. Existing source
contracts and the consumer handoff retain every lifecycle/preservation obligation.

| Baseline | Placement in v2 | Still required before native acceptance |
| --- | --- | --- |
| B01 — text/GFM | Add → Card; selected note Edit and double-click; local editable concept field | Full GFM and native selection/clipboard/IME, retained drafts, cancel, history and reopen. |
| B02 — document references/promotion | Visually distinct document cards with exact-target action; selection overflow → Promote | Define no new synchronization semantics. Actual target identity, explicit promotion, document editing and return-context evidence. |
| B03 — URL | URL card with source/domain; Add → Web link | Preview loading/failure, editing, deliberate authorized opening and original URL preservation. |
| B04 — images | Image object with selected resize grips; disclosed size property | Direct native resizing, aspect constraints/cancellation, original bytes, history and reopen. The concept size slider only demonstrates presentation. |
| B05 — PDF | Paper body, file identity and page controls; local synthetic page switch | Native decode/cache, page ownership at pan/zoom, stored page state and unavailable/corrupt recovery. |
| B06 — audio/video | Distinct media bodies, bounded transport and file labels | Actual native play/pause/seek, input ownership, lifetime, original bytes and reopen. No playback is claimed from this reference. |
| B07 — drop | Existing content families share Add destinations | Still design and verify OS/project drag previews, accurate zoomed placement, invalid/outside drop and cancellation. |
| B08 — ink | Visible pen stroke; grouped drawing entry | Native pen/highlighter/eraser tools, selection/erase semantics, cancel, history and exact retained strokes. |
| B09 — shapes | Rectangle, ellipse, diamond and drawn arrow; grouped shapes entry | All five original kinds including line; direct creation, re-editing, resize/rotation and style controls. |
| B10 — groups/frames | Named frames and nested group; Add and frame tool | Distinct move/content semantics, nesting, group lifecycle, frame presentation ordering and persistence. |
| B11 — transforms | Handles, image resize state, select/pan tools | Actual move, multi-select, resize, rotate, overlap hit rules and cancellation; concept cards do not implement dragging. |
| B12 — structured clipboard | Selection overflow with Copy/Cut/Duplicate | Paste and keyboard/menu/context equivalence, mixed-record bundles, fresh IDs, internal edges and resource mappings. |
| B13 — style/lock/layers | Typed/color variation; object overflow and Objects/Layers entry | Complete styles, lock feedback and enforcement, reorder/paint/hit agreement and reopen. |
| B14 — alignment | Selection overflow → Align/distribute | Full alignment/distribution/tidy property surface, locks, preview as applicable, cancel and one atomic history action. |
| B15 — connections/labels | Visible ports, labeled relations and consistent routing | Direct create/reconnect, selected-edge context/label editing, endpoint grips, invalid/pending/recovery, history and stable identity. V1 state detail remains a bounded reference, not a passed gesture. |
| B16 — types/schemas | Card type/status labels; Properties; board menu → Types and fields | Existing project-wide schema editor, custom choices/fields, compatible defaults and schema migration. Illustrative labels are not a new mandatory taxonomy. |
| B17 — templates | Add → From template | All fifteen original templates and their chooser/content/schema workflows remain required; not all illustrated. |
| B18 — views/filters | Board view menu names Free, Mind-map, Timeline, Swimlane, Dependency and filters | Design and verify each projection and filter, retained view state, exact canonical graph and safe return. Menu labels are not those view implementations. |
| B19 — layout | Board menu → Suggest layout | Disposable preview with separate Apply/Cancel, lock awareness, stale handling and exact geometry Undo. |
| B20 — navigation/diagnostics | Search, Objects/Layers and board-menu diagnostics | Search result/diagnostic panels, exact target reveal, keyboard navigation, missing-target behavior and focus return. |
| B21 — playthrough | Board menu → Check playthrough; dialogue/state examples | Full dialogue/quest/state traversal interfaces, branches, invalid transitions and non-mutating exit; not simulated by card labels. |
| B22 — presentation/timer | Board menu → Presentation and timer; named frames | Presentation/timer controls, progress/pause/exit and prior camera/focus restoration. |
| B23 — board lifecycle | Board menu → Rename/Duplicate/Trash; project trash | Complete valid/invalid/cancelled/failed lifecycle, restore, referenced targets, original/copy identities and history. |
| B24 — editable transfer | Board menu → Import/export .canvas | File dialogs, progress/errors, exact compatible data and editable round-trip; a rendered picture is not a backup. |
| B25 — output | Board menu → Image/PDF | PNG/SVG/PDF options, bounded export, mixed-media rendering, fonts/clipping, cancel and failure. |
| B26 — history/recovery | Shared history/shortcut expectation; input ownership remains explicit | Current pending/conflict/discard correction, text vs board history, durable save state, exact drafts, failures, retries and reopen. No Save-success state is invented in the concept. |

Supplemental source omissions reconciled by the root remain required:

- Other-board resource cards open the exact referenced board. Import/transfer
  remapping and missing/trashed targets need evidence; neither recursive embedded
  boards nor a dedicated source creation button is inferred. A convenient proposed
  Add location is presentation, not proof that the original used that location.
- External-file fallback remains an inert preserved path/reference. Unresolved
  records need honest presentation and authorized recovery; the engine does not
  gain arbitrary filesystem access.
- Canvas assistant context and guarded proposals/promotion retain an entry in the
  shared assistant. Source-backed ghost preview belongs to the **separate review
  page**, followed by explicit Apply/Reject/Undo. No on-board ghost overlay is
  claimed as verified baseline or made a new automatic requirement. This full
  review journey is not designed by the assistant icon alone.

### Review criteria updated after the breadth correction

Before accepting a consuming WriterCraft board, the independent designer must use
a mixed-content synthetic board comparable in breadth to v2, then complete the
applicable lifecycle of the active B slice. Inspect group/frame behavior alongside
normal cards; image sizing alongside actual PDF/audio/video control ownership;
shape/ink editing alongside edge selection; document/board/URL/file references
alongside missing-resource recovery. A text-only graph cannot qualify media,
drawings, custom schemas, views, export, assistant proposals or playthrough.

Ensure every original tool remains discoverable through the compact groups and
contextual routes. A reduction in permanent chrome must not move every ordinary
action into a distant command palette. Direct drag, double-click, selected-object
controls and conventional keyboard actions stay primary. Reopen and failed/stale
operations must retain data and drafts. The strict library issue order and exact
qualified application handoff are unchanged.

### V2 presentation verification

The v2 fragment was rendered in an isolated headless Chrome profile, without
opening or replacing either running native preview. Captures cover research and
story states in dark appearance, research in light appearance, selected image
properties, inline note editing and Add menu. A 320-pixel outer preview produced a
288-pixel inner reference; a 1024-pixel outer preview produced 992 inner pixels.
Both checked widths had no horizontal overflow. The research state contained nine
content objects and three canonical relation illustrations; the story state eight
objects and five relations. Native board spatial layout is not inferred from the
narrow reference reflow.

Local concept checks exercised selection, Properties, image reduction to 70%,
synthetic PDF page 2/3, switching board state, note editing and Escape. No JavaScript
exceptions were observed. Reference defects found during this check were fixed:
restoring preview state had cleared selection; relation labels were squeezed into
insufficient gaps; narrow reference reflow routed longer connections behind
intermediate cards. Narrow routes now use the side gutter. These are
reference-presentation repairs, not Rust bug fixes.

Files remain in the authorized conversation visualization directory:
`canvas-design-direction-v2.html`, `canvas-design-v2-preview.html` and
`canvas-design-v2-*.png`. The original `canvas-design-direction.html` is unchanged.
These are not application captures, user-validation sessions, a native gate or
proof of media playback/accessibility/persistence. Root owns publishing this
documentation and continuing CC-03; no Rust or application file was edited here.

Exact local reference receipt:

- `canvas-design-direction-v2.html` SHA-256:
  `fd8cfc0172ef9b149a5aecd26914472a73c64b88f4ba77ec2cb2375eaec0cb34`.
- `canvas-design-v2-checks.json` SHA-256:
  `110706a9205cb497668d8943f17464ac903b054272e9cdb70eee23d0f6577714`.

This receipt binds the inspected proposal only. The isolated renderer was stopped
after the checks; no user preview or existing browser profile was replaced.
