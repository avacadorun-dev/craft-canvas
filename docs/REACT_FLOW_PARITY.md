# React Flow functional parity inventory

2026-11-09 · **Capability coverage independently reviewed; implementation and native acceptance pending. No row is accepted.**

## Objective and frozen baselines

Prepare standalone CraftCanvas for production with full React Flow functional parity. The ordered delivery first qualifies every generic primitive required by current WriterCraft workflows and supplies an exact integration handoff, then completes remaining React Flow capabilities. Full WriterCraft Canvas acceptance remains a separate consumer obligation. Observable behavior and useful native extension points are required; React component names, DOM elements and JavaScript hook spelling are not the target. Work must not wait for upstream fixes or releases. CraftCanvas owns the implementation, maintenance and qualification schedule.

- **Upstream:** published `@xyflow/react 12.12.1`, released 2026-11-02; annotated tag object `081382f22a16743f243e9dd6177f0b39747a9b6b` resolves to **`1bc405cd113618c17fd181b931a9fe573bec9fee`**. Its package depends on `@xyflow/system 0.0.84`. The inspected source archive SHA-256 is `692fe75f0dc40c760919e4c9f88e5062732a7215251f15eba419272a33b5b2e9`. This is a frozen target; later upstream releases require an explicit inventory delta.
- **Current engine:** **`6b68eb4e84aa41b713a148e6a456048f3c1168f9`**, branch `feature/native-canvas-foundation`, egui `0.36.2`. Existing API/source findings below are not runtime or production acceptance.
- **Consumer baseline:** Studio declares `@xyflow/react ^12.11.2`; the installed `@xyflow/system 0.0.79` source hashes and existing geometry attribution are recorded in [upstream.md](upstream.md). Full standalone parity is broader than that compatibility baseline. [WRITERCRAFT_CONSUMER.md](WRITERCRAFT_CONSUMER.md) separately assigns all B01–B26 engine/host responsibilities.

Preserve one canonical document, raw unknown data, stable IDs and array order, exact-base guarded commands, bounded work/history and `f64` world geometry. Controlled and convenience modes must share that authority. Native integration must not introduce a shadow JavaScript-style graph, ambient resource access or a browser runtime. React Flow serialization defaults are not permission to rewrite existing JSON Canvas records.

## Source catalogue and interpretation

Every `RF-*` row is a required capability target. **Core** means exported upstream package behavior; **Composition** means a public example or documented integration built from those primitives; **Pro-reference** means a publicly described behavior whose premium implementation was not inspected. Classification explains provenance, not a scope exemption. A missing implementation remains work. Native-equivalent decisions below require independent review, particularly where browser machinery has no direct native counterpart.

Immutable primary sources, all at the upstream commit above:

| Key | Inspected surface |
|---|---|
| [API](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/index.ts) | Package exports, including hooks, experimental middleware and system utilities. |
| [PROPS](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/types/component-props.ts) | Complete `ReactFlowProps`: input policies, events, connection/viewport options, accessibility and appearance. |
| [NODE](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/system/src/types/nodes.ts) / [NODE-UI](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/types/nodes.ts) | Node data, dimensions, origins, parent relationships, policy flags and custom-node wrapper. |
| [HANDLE](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/system/src/types/handles.ts) / [CONNECT](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/system/src/xyhandle/XYHandle.ts) | Named ports, directional policies, creation, candidate validation and connection lifecycle. |
| [EDGE](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/system/src/types/edges.ts) / [EDGE-UI](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/types/edges.ts) | Edge identity/data, markers, geometry, labels, custom routes, reconnect policies and input width. |
| [INSTANCE](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/types/instance.ts) / [GENERAL](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/types/general.ts) | Read/update/query/serialization and viewport helper contracts. |
| [WIDGETS](https://github.com/xyflow/xyflow/tree/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/additional-components) | Background, Controls, MiniMap, NodeResizer/NodeResizeControl, NodeToolbar, EdgeToolbar and their complete option types. |
| [PROVIDER](https://github.com/xyflow/xyflow/blob/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/components/ReactFlowProvider/index.tsx) / [HOOKS](https://github.com/xyflow/xyflow/tree/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/react/src/hooks) | Initialization, multiple contexts, state observations, cancellation and middleware. |
| [SYSTEM](https://github.com/xyflow/xyflow/tree/1bc405cd113618c17fd181b931a9fe573bec9fee/packages/system/src) | XYDrag, XYPanZoom, XYHandle, XYResizer, XYMinimap; graph/path/connection utilities and accessibility constants. |
| [EXAMPLES](https://github.com/xyflow/xyflow/tree/1bc405cd113618c17fd181b931a9fe573bec9fee/examples/react/src/examples) | The complete versioned React example directory; explicit coverage crosswalk below. |

Supplementary official documentation is dated **2026-11-09**, not treated as an immutable implementation snapshot: [example catalogue](https://reactflow.dev/examples), [accessibility](https://reactflow.dev/learn/advanced-use/accessibility), [SSR](https://reactflow.dev/learn/advanced-use/ssr-ssg-configuration), [Undo/Redo example](https://reactflow.dev/examples/interaction/undo-redo). Public Pro descriptions establish intended behavior; they do not prove inspected source or grant a right to copy it.

Current status shorthand: **Partial** = some relevant engine mechanism exists, with the stated gap; **Gap** = no adequate exported implementation found at the engine baseline; **Host** = a generic native extension point and a runnable consumer demonstration are required, while authorization/I/O remain outside the engine. These are static findings. Evidence codes refer to grouped native journeys below; they are not a demand for a test per property.

## Required capability inventory

### Data, custom nodes and handles

| ID | Kind; upstream source | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-001 | Core; NODE, EDGE, INSTANCE | Stable node/edge identity, extensible typed data, type discriminators, lookup, merge/replace updates and default options; preserve unknown fields and numeric precision through edits. | **Partial:** preserved raw records and `SetNodeFields`/`SetEdgeFields` exist; generic React-style type/default/update surface is incomplete. J1, J7. |
| RF-002 | Core; NODE-UI, API | Built-in default/input/output/group equivalents and a custom-node registry, including replacement of built-ins and changing a node type without losing identity/data. | **Partial:** fixed JSON Canvas kinds and host body paint exist; custom outer geometry/wrapper/registry is missing. J1. |
| RF-003 | Core; NODE-UI, PROPS | Real embedded native text, buttons, scroll areas and media controls; explicit no-drag/no-pan/no-wheel regions, focus/IME ownership and input bubbling without double actions. | **Partial:** child pointer/keyboard ownership and `TextCardEditor` exist; wheel/touch ownership and general gesture arbitration remain incomplete. J1, J6. |
| RF-004 | Core; NODE, PROVIDER, HOOKS | Explicit and measured size, initial size, origin, absolute/local position and dynamic measurement invalidation; update handle geometry after content/size changes. | **Partial:** explicit rectangles only; no generic measured dimensions/origin/recompute lifecycle. J1, J3. |
| RF-005 | Core; NODE, PROPS | Per-node and global draggable/selectable/connectable/deletable/focusable/hidden policies, drag regions and read-only rendering; temporary dragging/resizing state must be observable. | **Partial:** movable/resizable/connectable and global interaction gates exist; remaining independent policies are absent. J1, J3, J6. |
| RF-006 | Core; NODE, SYSTEM | Nested parent/subflow relationships, relative transforms, parent extent constraints, expand-parent behavior and origin-aware child movement. | **Partial:** spatial group/frame expansion exists, but it is not persistent `parentId` hierarchy or relative-coordinate semantics. J3, J7. |
| RF-007 | Core; NODE, EDGE, PROPS | Predictable node/edge z-order, selection elevation, parent ordering and `zIndexMode` behavior, consistent between paint, hover, hit testing and focus. | **Partial:** canonical node order/reorder and reverse rectangle hits exist; independent z policies and edge stacking are missing. J2, J3. |
| RF-008 | Core; HANDLE, NODE | Multiple stable named source/target handles, including several on one side, custom positions/sizes, detached and dynamically added/removed handles. | **Gap:** `ConnectorEndpoint` carries only node plus optional side. Requires compatible port identity/geometry extension; a side is not a unique port. J1, J2. |
| RF-009 | Core; HANDLE, PROPS | Independent connectable/start/end policies per handle, strict/loose mode and direction rules; retain same-card distinct-handle capability where valid. | **Partial:** node-level `can_connect` and side loops exist; port-specific directional policy does not. J2. |
| RF-010 | Core; HANDLE, HOOKS, INSTANCE | Observe live/pending/final connections; query node/handle connections filtered by type/ID; dynamically invalidated ports cancel frozen gestures safely. | **Gap:** no generic connection-state/port query or measurement-revision contract. J2, J7. |

### Creation, reconnection and edge presentation

| ID | Kind; upstream source | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-011 | Core; CONNECT, PROPS | Direct pointer/touch handle drag creation with configurable start threshold and target radius, hover/selected handles and deterministic valid target selection. | **Gap:** model insertion exists; engine has no connection drag state. J2. |
| RF-012 | Core; CONNECT, PROPS | Optional click-to-connect alongside primary dragging; clear start/end state and programmatic cancellation. Escape/focus/pointer-loss cancellation leaves canonical content unchanged. Invalid/outside release also preserves by default; explicit host drop policies may instead prepare RF-055/070/072 commands. | **Gap:** host two-click UI is not standalone engine equivalence. Existing generic cancellation does not cover the missing gesture. J2, J6. |
| RF-013 | Core; EDGE-UI, PROPS | Drag either existing endpoint to another handle; configure source-only/target-only/both/disabled reconnect globally and per edge. Keep the original edge until successful commit; cancellation restores its exact identity/data. | **Partial:** exact `ReconnectEdge` command exists; no reconnect gesture, endpoint affordances or restriction policy. J2, J7. |
| RF-014 | Core; HANDLE, CONNECT, PROPS | Candidate-level global and per-handle validation with valid/invalid feedback; graph-aware policies support connection limits, type compatibility and cycle prevention without mutation. | **Partial:** whole-document validation/node connectability exist; preview-time candidate validation is missing. J2, J8. |
| RF-015 | Core; EDGE-UI, PROPS | Customizable connection preview: all standard route types, source/target geometry, candidate status and user-drawn native preview content. | **Gap:** no public connection-preview renderer/state. J2. |
| RF-016 | Core; PROPS, CONNECT | Ordered connect/reconnect start, progress/end/cancel observations, including invalid/no-op outcomes, with exact source, candidate, final status and edge identity; expose one atomic completion intent. | **Gap:** existing completed `CanvasIntent` lacks this lifecycle. J2, J7. |
| RF-017 | Core; API, EDGE, SYSTEM | Straight, step, smooth-step, Bézier and simple-Bézier routes; route parameters including curvature, offset, corner radius and step position; reusable path/label-center utilities. | **Partial:** model cubic and rounded smooth-step helpers exist; the adapter uses one route and lacks full style/parameter selection. J2, J4. |
| RF-018 | Core; EDGE-UI, API | Custom edge renderer with shared route/hit geometry, native controls and arbitrary label placement; floating/perimeter/custom-shape routes supported through extension points. | **Partial:** emitted route geometry/label anchors exist; no general edge renderer or input-ownership callback. J2, J4. |
| RF-019 | Core; EDGE, SYSTEM | Start/end markers: open/closed arrows and custom markers, size, color, stroke/orientation and scaling; no rewrite of omitted imported defaults. | **Partial:** none/arrow triangles render; full marker/style customization is absent. J4, J7. |
| RF-020 | Core; EDGE-UI, API | Text and custom native edge labels, padding/background/radius/style, interaction-safe label surfaces and direct editing integration. | **Partial:** label data/command and geometry anchors exist; WriterCraft's label widget is not a reusable standalone engine facility. J2, J4. |
| RF-021 | Core; EDGE, PROPS | Direct path click/double-click/hover/context actions, configurable hit width, edge focus/selection/deletion and independent hidden/selectable/deletable/reconnectable flags. | **Partial:** shared route hit testing and edge Delete exist; full flags/events/accessibility are missing. J2, J6. |
| RF-022 | Core; EDGE, EDGE-UI | Per-edge styling, animation, hidden state, selected appearance, type defaults and live changes without identity loss. | **Partial:** raw fields survive; most are not interpreted/rendered. Preservation alone does not pass. J4, J7. |
| RF-023 | Core; API, INSTANCE | Add/reconnect helpers with explicit duplicate/parallel-edge policy and merge behavior. Any requested ID replacement needs an explicit mapping; imported parallel edges and stable native IDs must not disappear through convenience defaults. | **Partial:** insert/reconnect preserve IDs; generic compatibility helper/default policy remains to specify. J2, J7. |

### Selection, movement, resizing and viewport

| ID | Kind; upstream source | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-024 | Core; PROPS, SYSTEM | Single/additive/toggle selection, marquee full/partial inclusion, selection-on-drag and selectable/read-only policies; selection rect/start/end and context actions. | **Partial:** single/additive/marquee selection exists; configurable inclusion/modes and lifecycle are incomplete. J3. |
| RF-025 | Core; PROPS, SYSTEM | Drag single/multiple selected nodes with thresholds, start/move/stop events, stable offsets and coherent cancellation across nested groups. | **Partial:** one atomic translate gesture exists; configurable lifecycle, nested parent transforms and outside/pointer-loss handling need work. J3, J7. |
| RF-026 | Core; NODE, PROPS | Configurable snap grid, node origin, drag region, node/parent extents and allowed coordinates; distinguish click from drag at configurable distances. | **Partial:** current drag rounds geometry and uses Studio-oriented limits; explicit generic profiles and optional snapping are required. J3. |
| RF-027 | Core; WIDGETS, SYSTEM | NodeResizer and custom resize controls: all corners/sides/line variants, direction restrictions, min/max dimensions, aspect ratio, zoom scaling and conditional resize/lifecycle callbacks. | **Partial:** one bottom-right resize gesture and geometry command; remaining controls/policies absent. J3. |
| RF-028 | Core; PROPS, INSTANCE | Deletion with per-object flags, incident-edge handling, before-delete defer/veto/transform and precise completion callbacks against a frozen base. | **Partial:** removal/incident edges are guarded; asynchronous preflight and policy/event surface are missing. J3, J7. |
| RF-029 | Core; PROPS, SYSTEM | Configurable pointer-button/key pan, wheel scroll/pan modes/speed, zoom modifier, zoom-on-scroll/pinch/double-click and propagation/scroll prevention. | **Partial:** middle/Space pan, smooth scroll and zoom exist; configurable parity and child wheel ownership are incomplete. J3, J6. |
| RF-030 | Core; PROPS, SYSTEM | Touch drag, tap/click connection, pinch zoom, pan and cancellation with usable hit targets and embedded touch-scroll arbitration. | **Gap:** no complete engine touch policy or device evidence. egui support alone is not acceptance. J6. |
| RF-031 | Core; PROPS, SYSTEM | Auto-pan while dragging nodes, connecting or selecting near viewport bounds, configurable speed; focus-based auto-reveal. | **Gap:** no generic auto-pan/focus policy. J2, J3, J6. |
| RF-032 | Core; INSTANCE, GENERAL, PROPS | Controlled/default viewport, min/max zoom, translation extents, screen/world transforms and zoom anchored at pointer. | **Partial:** `f64` camera/transforms exist; configurable bounds/zoom profiles are incomplete. J3, J7. |
| RF-033 | Core; INSTANCE, GENERAL | Fit all/subset/hidden-inclusive nodes, fit bounds, set center, zoom in/out/to, set/get viewport; padding and animated duration/easing/interpolation. | **Partial:** camera set/fit exists; complete helper options and animated transitions absent. J3, J4. |
| RF-034 | Core; PROPS, HOOKS | Visibility/culling, explicit hidden state, initialized viewport/node measurements and first-frame readiness; painted and hittable objects agree. | **Partial:** visible card/edge output exists; hidden/measurement/readiness policies and same-frame camera consistency require qualification. J1, J3. |

### Reusable native widgets and customization

| ID | Kind; upstream source | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-035 | Core; WIDGETS | Background dots/lines/cross, separate X/Y spacing/offset, color/size/stroke and multiple layers. | **Partial:** fixed dotted grid only. J4. |
| RF-036 | Core; WIDGETS | Controls with zoom/fit/interaction toggles, custom controls, placement/orientation, callbacks and accessible labels. | **Gap:** hosts assemble buttons; no reusable complete control widget. J4, J6. |
| RF-037 | Core; WIDGETS, SYSTEM | MiniMap with viewport mask, custom node representation/colors, click/node-click, pan/zoom/inverse-pan/speed/options and accessibility. | **Gap.** J4, J6. |
| RF-038 | Core; WIDGETS | Node toolbar including multiple-node anchors, alignment/offset/visibility and zoom-independent native controls. | **Gap.** J4. |
| RF-039 | Core; WIDGETS | Edge toolbar at derived/custom position, alignment/visibility, correct zoom and input ownership. | **Gap.** J2, J4. |
| RF-040 | Core; API | Fixed viewport panels, flow-coordinate overlays/portals and native label layers; clipped, z-ordered, independently interactive surfaces. | **Partial:** host painting and route geometry exist; no reusable panel/overlay ownership contract. J1, J4. |
| RF-041 | Core; PROPS, NODE-UI, EDGE-UI | Light/dark/system appearance; configurable theme/style/hover/focus/selection states, node/edge-specific presentation and host-localized labels. DOM classes/attributes map to native style, semantics and interaction metadata. | **Partial:** host theme/basic labels exist; complete state/style and accessibility metadata surfaces are missing. J4, J6. |

### State, instance operations and observations

| ID | Kind; upstream source | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-042 | Core; PROVIDER, API, HOOKS | Externally controlled flow plus convenient in-memory ownership, initialization/remount and independent multi-flow contexts. Both use one canonical authority with explicit ownership. | **Partial:** controlled session/view split exists; standalone convenience/provider lifecycle needs demonstration. J1, J7. |
| RF-043 | Core; INSTANCE, API | Get/add/update/replace/delete nodes/edges/data individually or in batches, retaining type/identity rules and exact host-controlled change acknowledgement. | **Partial:** transactional insert/fields/remove exist; full bulk/replace helper contract absent. J7. |
| RF-044 | Core; API, HOOKS, SYSTEM | Apply node/edge change batches including position/dimensions/selection/add/remove/replace, plus experimental change interception/middleware. Native middleware must preserve atomicity and stale checks. | **Partial:** guarded transaction batches exist; typed change-set observation/interception does not. J7. |
| RF-045 | Core; PROPS | Complete node/edge/pane click/double-click/context/enter/move/leave; drag, selection, connect/reconnect, delete and viewport lifecycle events, initialization and errors. Event order must be documented and composable. | **Partial:** semantic completion intents and errors exist; full observation surface is absent. J1–J3, J7. |
| RF-046 | Core; HOOKS, API | Observe node/edge lists, node data, internal geometry, local node identity, viewport/selection changes, initialization, connection progress and key state; selector subscriptions without unnecessary full-graph copying. | **Partial:** immutable getters, current selection/camera and frame geometry exist; subscriptions/query granularity absent. J1, J7. |
| RF-047 | Core; INSTANCE, API, SYSTEM | Bounds, partial/full intersections, node/edge type guards, incoming/outgoing neighbors, connected edges and handle-filtered node connections; pure path/viewport utilities. | **Partial:** rectangle operations, document bounds and ID lookup exist; complete graph/spatial query utilities absent. J3, J7. |
| RF-048 | Core; INSTANCE, EXAMPLES | Save/restore nodes, edges and viewport with deterministic no-edit/semantic round trips; restore before mounting, remount cleanly and recover unsupported fields. | **Partial:** raw document round trip/session exists; viewport bundle and complete native restore journey require host integration. J7. |
| RF-049 | Core; PROPS, SYSTEM | Error callbacks and debug diagnostics identify missing types/handles, invalid relationships/geometry and failed operations; no silent fallback reported as success. | **Partial:** typed model errors/view error output exist; full diagnostics and actionable standalone UI are incomplete. J7. |
| RF-050 | Core; PROPS, SYSTEM | Culling and stable interaction for large graphs, bounded route/query work, numerical stability, interruption/cancellation and isolation between flows. | **Partial:** explicit count/byte/history/work limits and relative-origin precision exist; generic policy profiles, profiling and production thresholds remain unaccepted. J3, J7. |

### Keyboard, accessibility and browser-independent equivalents

| ID | Kind; upstream source | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-051 | Core; PROPS, HOOKS, SYSTEM | Configurable Delete/selection/multiselect/pan/zoom key bindings; Tab navigation, Enter/Space selection, arrow/Shift movement and Escape cancellation obey input ownership and read-only policy. | **Partial:** fixed board shortcuts and native text ownership exist; configurable complete board/edge/handle keyboard path absent. J6. |
| RF-052 | Core; NODE-UI, EDGE-UI, PROPS | Accessible roles/names, per-node/edge focusability, localized live guidance/position announcements, focus auto-reveal and disable/re-enable policy. Native AccessKit/platform equivalents must expose actual actions and focus, including handle operations. | **Partial:** node WidgetInfo and text-editor AccessKit work exist; edge/handle semantics, announcements and complete traversal are missing. J6. |
| RF-053 | Core; PROVIDER, NODE, official SSR documentation | Native SSR/SSG equivalent: deterministic headless initial geometry/rendering using supplied viewport, dimensions and handles, followed by interactive mounting without data/position drift. No DOM or server process is required merely to preserve this capability. | **Partial:** headless model/geometry exists; detached render scene and full first-frame/hydration-equivalent demonstration are absent. J4, J7. |
| RF-054 | Core; PROPS, NODE-UI, EDGE-UI | Framework escape hatches (`domAttributes`, style/classes, attribution/pro options) map to native semantics, styling and host composition. Document browser-only implementation details and their observable replacement; do not silently mark features N/A. | **Host/Gap:** native replacement contract and provenance/attribution configuration need review. J4, J6. |

### Official compositions and additional behavior

These rows retain the broader requested capability scope. A composition is not falsely described as a built-in React Flow command, and a missing Pro source is not treated as implemented or excluded.

| ID | Kind; primary reference | Native equivalent and required behavior | Current engine finding; required evidence |
|---|---|---|---|
| RF-055 | Composition; EXAMPLES `DragNDrop`, `AddNodeOnEdgeDrop` | Palette/external node drag/drop, screen-to-world placement and optional create-node-on-valid/invalid-edge-drop policy. Cancellation never implicitly creates/deletes objects. | **Partial/Host:** `CreateText` and insert commands exist; generic DnD and drop composition absent. J1, J2, J8. |
| RF-056 | Composition; EXAMPLES `EasyConnect`, `FloatingEdges`, `Undirectional`, `MovingHandles`, `DetachedHandle`, `CustomConnectionLine` | Easy/body-target connection, floating/perimeter routes, undirected presentation, moving/detached ports and custom temporary lines through explicit policies. | **Gap:** route geometry is useful but does not implement these interaction policies. J2, J4. |
| RF-057 | Composition; EXAMPLES `Validation`, `UseNodeConnections`, `UseNodesData`; official computing-flow/cycle/connection-limit examples | Connection limits, cycle prevention and reactive data-processing demonstrations through graph queries/validators; application computations remain host-supplied. | **Host/Gap:** generic APIs and representative native compositions missing. J2, J8. |
| RF-058 | Composition/Pro-reference; official Copy/Paste and Undo/Redo examples | Structured copy/cut/paste/duplicate with fresh IDs/internal-edge remapping, exact selection, coherent Undo/Redo, drag coalescing and saved-state restoration. Clipboard I/O remains capability-bound to the host. | **Partial:** history/duplicate and clipboard intents exist; standalone native clipboard and complete functional acceptance missing. J7, J8. |
| RF-059 | Composition; EXAMPLES `Layouting`; official Dagre, ELK, tree, force/auto-layout examples | Pluggable deterministic/bounded layout, async cancellation and measured/port-aware placement; explicit preview/apply where a host requires approval. Supply equivalent native layout demonstrations without making a JS library a runtime requirement. | **Gap/Host:** no layout integration/result contract. J3, J8. |
| RF-060 | Composition/Pro-reference; EXAMPLES `Subflow`, `Intersection`; official grouping, expand/collapse and collision examples | Group/ungroup/reparent/detach, intersections, containment, expand/collapse and collision-aware layouts, preserving child identities/positions and hidden descendants. | **Partial:** spatial group expansion is insufficient; hierarchy operations/compositions missing. J3, J8. |
| RF-061 | Composition/Pro-reference; official helper-line and rotatable-node examples | Alignment/snapping guides, resizable/rotatable custom geometry and transform-aware handles/routes; guides remain transient until a guarded edit. | **Gap:** no guide/rotation contract; one-corner resize is partial. J3, J8. |
| RF-062 | Composition/Pro-reference; official whiteboard examples | Freehand, rectangle/shape drawing, lasso and eraser interactions as reusable native tools with bounded paths, selection and undoable operations. | **Gap:** preserved unknown drawing data is not authored drawing. J3, J8. |
| RF-063 | Composition/Pro-reference; official editable/animated/multi-connection edge examples | Editable route control points, animation and multiple-connection previews as custom-edge/gesture compositions with stable identity and direct manipulation. | **Gap:** generic extension points and runnable equivalents missing. J2, J4, J8. |
| RF-064 | Composition; official image-download/server-image examples; SSR documentation | Offscreen/exportable scene and images using the same geometry, text/asset policy and clipping as native display; deterministic target bounds and no canonical mutation. | **Gap/Host:** headless geometry exists; reusable export display list and renderer missing. J4, J7. |
| RF-065 | Composition/Pro-reference; official collaborative example and multiplayer guidance | Externally synchronized changes, multiple cursors/selections and conflict-aware updates through an authority-safe integration. Network/account/backend choices stay with hosts. | **Host/Gap:** exact-base refusal is a prerequisite; collaborative projection/merge demonstration is missing. J7, J8. |
| RF-066 | Composition; EXAMPLES `DevTools`, official React Flow UI/templates | Inspectable state/geometry/events and reusable native custom-node/control compositions sufficient for workflow-style editors. Template aesthetics are not a substitute for interactions; public Pro implementation remains uninspected. | **Partial/Host:** frame geometry exists; standalone inspector/demo and composition catalogue incomplete. J1, J4, J8. |
| RF-067 | Composition; [Proximity Connect](https://reactflow.dev/examples/nodes/proximity-connect) | While moving a node, preview the nearby-node connection that an opted-in drop will create; apply the move/connection coherently with duplicate and validation policy. | **Gap:** nearest-target preview and drop composition absent. J2, J3, J8. |
| RF-068 | Pro-reference; [Node Position Animation](https://reactflow.dev/examples/nodes/node-position-animation) | Animate node positions between layouts with coherent attached routes and hit geometry, interruption and an explicit canonical completion boundary; this is separate from viewport animation. | **Gap:** no node/layout transition mechanism. J3, J4, J8. |
| RF-069 | Composition; [Delete Middle Node](https://reactflow.dev/examples/nodes/delete-middle-node) | Opted-in deletion of `a→b→c` can reconnect surviving predecessors/successors as `a→c`, with deliberate ID/duplicate policy and one guarded undoable transaction. | **Partial:** insert/remove primitives exist; neighbor queries and complete native composition absent. J7, J8. |
| RF-070 | Composition; [Delete Edge on Drop](https://reactflow.dev/examples/edges/delete-edge-on-drop) | An explicit host policy may turn a reconnect release on the pane into an undoable edge deletion. Distinguish an intentional drop from Escape, focus loss, stale state or interruption; ordinary cancellation retains the original. | **Gap:** no reconnect-end/drop policy. This optional composition must not change WriterCraft's preserve-on-cancel default. J2, J7, J8. |
| RF-071 | Composition; [Edge Intersection](https://reactflow.dev/examples/edges/edge-intersection) | Detect a dragged node intersecting the rendered edge's interaction area and expose feedback/query results. The cited example does not establish an insert/split topology mutation. | **Gap:** point-edge hit testing exists, but reusable node/edge intersection queries and the native composition do not. J3, J8. |
| RF-072 | Composition; [Temporary Edges](https://reactflow.dev/examples/edges/temporary-edges) | Explicitly create and manage a ghost-node/incomplete-edge composition after a connection drop, then reconnect to complete it and remove the placeholder, or delete it coherently. A transient preview alone is insufficient. | **Gap:** no incomplete-connection lifecycle/example; canonical placeholder and recovery semantics must be specified. J2, J7, J8. |
| RF-073 | Composition; [Contextual Zoom](https://reactflow.dev/examples/interaction/contextual-zoom) | Change custom-node detail/content with zoom while preserving data, focus/drafts and stable sizing/port semantics; demonstrate level-of-detail behavior beyond scaling the same text. | **Partial/Host:** body renderer receives zoom; no complete native composition/input qualification. J1, J3, J8. |
| RF-074 | Pro-reference; [Edge Routing](https://reactflow.dev/examples/edges/edge-routing) | Obstacle-avoiding automatic routing equivalent to the documented libavoid composition, with bounded/cancelable recalculation as geometry changes. A custom path callback or simple orthogonal route alone does not pass. | **Gap:** existing smooth-step geometry does not avoid arbitrary obstacles. Public behavior was inspected; Pro implementation was not. J2, J4, J8. |

## Ordered delivery allocation

[ROADMAP.md](ROADMAP.md) owns the actual issues and strict CC-01 → CC-14 order. Complete and verify one issue before implementing the next. CC-02–CC-12 establish the **WriterCraft-ready engine** checkpoint; CC-13 completes remaining full-parity behavior and CC-14 accepts production readiness. This allocation does not authorize concurrent engine implementation or claim that a checkpoint completes WriterCraft #17. Separately coordinated non-Canvas application work may proceed without adopting an unqualified engine.

Each RF ID has exactly one **primary delivery step** below. **W** identifies a source-backed consumer primitive, using [B01–B26 and WC-ARCH contracts](WRITERCRAFT_CONSUMER.md); it does not claim every upstream option existed in Studio. **M** is a mixed row whose required consumer subset must ship earlier, with the broader API/composition completed in CC-13. **R** identifies remaining standalone behavior without a demonstrated requirement for the current consumer checkpoint. Some reusable controls are scheduled early for the library even where no individual Studio requirement is established.

| Primary step | RF IDs | Required consumer boundary / broader work |
|---|---|---|
| CC-02 | RF-008, RF-009, RF-014, RF-023 | **W:** B15/B26 connection identity, valid endpoints and preserved legacy defaults. Named/dynamic ports generalize that foundation under WC-ARCH-01; their full generic semantics are not attributed to Studio's four-side UI. |
| CC-03 | RF-011, RF-012, RF-013, RF-015, RF-016 | **W:** B15/B26 direct connection gestures, preview, validation, lifecycle and cancellation. Optional click-connect remains a standalone capability; it cannot replace primary dragging. |
| CC-04 | RF-004, RF-005, RF-006, RF-007, RF-024, RF-025, RF-026, RF-027 | **W:** B04/B10/B11/B13 geometry, policies, selection and transforms. Distinguish Studio spatial groups/frames from the additional persistent-parent semantics. |
| CC-05 | RF-002, RF-017, RF-018, RF-019, RF-020, RF-021, RF-022, RF-041 | **W:** B01–B06/B13/B15/B16 interactive bodies, edge/label rendering and type/style hooks. Full custom registries, route/style options and animation remain explicit parts of these rows. |
| CC-06 | RF-029, RF-031, RF-032, RF-033, RF-034, RF-035, RF-036, RF-037, RF-038, RF-039, RF-040 | **W:** B11/B18/B20/B22 navigation, reveal, shared geometry and bounded overlays. Complete background variants, minimap and reusable node/edge toolbars are scheduled library capabilities; this table does not invent a matching Studio feature for each control. |
| CC-07 | RF-062 | **W:** B08/B09/B10/B11/B13 strokes, shapes, frames and erase/transform history. The additional lasso composition remains required by the full row. |
| CC-08 | RF-053, RF-058, RF-059, RF-064 | **W:** B12/B14/B19/B24/B25/B26 structured operations, guarded layout and detached output. General headless initialization and the full layout demonstration family extend those consumer primitives. |
| CC-09 | RF-003, RF-051, RF-052 | **W:** B01/B05/B06/B11/B15/B26 embedded input, keyboard and actual native accessibility. Implement ownership from CC-03/CC-05 onward; this step qualifies the integrated contract. Unavailable OS/device evidence stays pending. |
| CC-10 | RF-050 | **W:** all consumer workflows depend on bounded, responsive operation at retained source capacities; full standalone workloads and supported-platform claims require their own measurements. |
| CC-11 | — | Packaging, API documentation, provenance and external-consumer verification cover all delivered rows; no unique behavioral RF ID is assigned to packaging. |
| CC-12 | — | Independently audit every B01–B26 engine prerequisite and deliver the exact qualified version. This is a cross-cutting handoff gate, not another behavioral row. |
| CC-13 | RF-001, RF-010, RF-028, RF-042, RF-043, RF-044, RF-045, RF-046, RF-047, RF-048, RF-049, RF-055, RF-057, RF-060, RF-061 | **M:** complete the broader state/query/event/middleware, deferred-delete, drop, computation and grouping/guide surfaces after the mandatory consumer subsets below are qualified. |
| CC-13 | RF-030, RF-054, RF-056, RF-063, RF-065, RF-066, RF-067, RF-068, RF-069, RF-070, RF-071, RF-072, RF-073, RF-074 | **R:** complete general touchscreen evidence, remaining framework equivalents and the named standalone compositions. Their primitives may be implemented earlier; none is silently waived. |

For **M** rows, the following are CC-12 prerequisites, not deferred work: stable raw data/IDs and transactional CRUD/history/restore (RF-001/043/044/048, B12/B23/B24/B26); port state and actual gesture/events/diagnostics (RF-010/045/046/049, CC-02–CC-06); policy-respecting deletion (RF-028, CC-04); lookup/bounds/graph navigation (RF-047/057, B15/B20/B21); authorized external/project drop geometry and atomic insertion (RF-055, B07); source-required grouping, transforms, rotation and layout (RF-060/061, CC-04/CC-08). RF-042 must already demonstrate one canonical authority and a reusable standalone host. Broader provider/multiflow, subscription/middleware, deferred-delete, create-on-edge-drop, reactive computation, collision and helper-guide compositions remain with their primary CC-13 owner. Clipboard/storage/network/media authorization remains host-owned throughout.

A row assigned to an early step is accepted only when its **entire** stated behavior and required evidence pass. If that step qualifies only its consumer subset, record the row as partial and explicitly list its remainder in CC-13 before the CC-12 handoff; do not relabel the partial row complete. CC-13 re-audits all 74 rows, including these carried remainders. CC-14 requires complete row acceptance plus the production gates. This keeps the first useful checkpoint bounded without reducing the full target.

## Public API coverage crosswalk

This source audit identifies **122 directly declared `ReactFlowProps` members** and **66 versioned example directories**. These counts exclude inherited HTML attributes and do not constitute acceptance. Inherited browser attributes are accounted for by RF-054; exported data types/enums express the corresponding row's data/options contract rather than a requirement to duplicate TypeScript syntax.

| Exported surface | Required rows |
|---|---|
| `ReactFlow`, `ReactFlowProvider` | RF-001–005, RF-042, RF-053 |
| `Handle` | RF-008–016 |
| `BaseEdge`, `StraightEdge`, `StepEdge`, `BezierEdge`, `SimpleBezierEdge`, `SmoothStepEdge`, `EdgeText`, `EdgeLabelRenderer` | RF-017–023, RF-040 |
| `Panel`, `ViewportPortal`, `Background`, `BackgroundVariant`, `Controls`, `ControlButton`, `MiniMap`, `NodeResizer`, `NodeResizeControl`, `NodeToolbar`, `EdgeToolbar` | RF-027, RF-035–041 |
| `useReactFlow`, `useStore`, `useStoreApi`, `useNodes`, `useEdges`, `useNodesState`, `useEdgesState`, `useNodesData`, `useInternalNode`, `useNodeId` | RF-042–047 |
| `useViewport`, `useOnViewportChange`, `useOnSelectionChange`, `useNodesInitialized`, `useUpdateNodeInternals`, `useKeyPress` | RF-004, RF-032–034, RF-045–046, RF-051 |
| `useConnection`, `useNodeConnections`, deprecated `useHandleConnections` | RF-010, RF-014–016, RF-046–047 |
| `experimental_useOnNodesChangeMiddleware`, `experimental_useOnEdgesChangeMiddleware`, `applyNodeChanges`, `applyEdgeChanges` | RF-044; experimental upstream status remains explicit, not omitted |
| `isNode`, `isEdge`, `getNodesBounds`, `getIncomers`, `getOutgoers`, `getConnectedEdges`, `getViewportForBounds` | RF-033, RF-047 |
| `getSimpleBezierPath`, `getBezierEdgeCenter`, `getBezierPath`, `getEdgeCenter`, `getSmoothStepPath`, `getStraightPath` | RF-017–018, RF-047 |
| `addEdge`, `reconnectEdge` | RF-013, RF-023, RF-043 |
| Instance `getNodes`, `setNodes`, `addNodes`, `getNode`, `getInternalNode`, `getEdges`, `setEdges`, `addEdges`, `getEdge`, `updateNode`, `updateNodeData`, `updateEdge`, `updateEdgeData`, `deleteElements`, `toObject` | RF-028, RF-043–048 |
| Instance `getIntersectingNodes`, `isNodeIntersecting`, `getNodesBounds`, `getHandleConnections`, `getNodeConnections` | RF-010, RF-047 |
| Instance `zoomIn`, `zoomOut`, `zoomTo`, `getZoom`, `setViewport`, `getViewport`, `setCenter`, `fitBounds`, `fitView`, `screenToFlowPosition`, `flowToScreenPosition`, `viewportInitialized` | RF-032–034, RF-046 |

Complete direct `ReactFlowProps` grouping at the frozen PROPS source:

| Property/event names | Required rows |
|---|---|
| `nodes`, `edges`, `defaultNodes`, `defaultEdges`, `defaultEdgeOptions`, `nodeTypes`, `edgeTypes`, `onInit`, `width`, `height` | RF-001–004, RF-018, RF-042–043, RF-053 |
| `onNodeClick`, `onNodeDoubleClick`, `onNodeMouseEnter`, `onNodeMouseMove`, `onNodeMouseLeave`, `onNodeContextMenu`, `onNodeDragStart`, `onNodeDrag`, `onNodeDragStop` | RF-003, RF-025, RF-045 |
| `onEdgeClick`, `onEdgeContextMenu`, `onEdgeMouseEnter`, `onEdgeMouseMove`, `onEdgeMouseLeave`, `onEdgeDoubleClick` | RF-021, RF-045 |
| `onReconnect`, `onReconnectStart`, `onReconnectEnd`, `onConnect`, `onConnectStart`, `onConnectEnd`, `onClickConnectStart`, `onClickConnectEnd` | RF-011–016, RF-045 |
| `onNodesChange`, `onEdgesChange`, `onNodesDelete`, `onEdgesDelete`, `onDelete`, `onBeforeDelete` | RF-028, RF-043–045 |
| `onSelectionDragStart`, `onSelectionDrag`, `onSelectionDragStop`, `onSelectionStart`, `onSelectionEnd`, `onSelectionContextMenu`, `onSelectionChange` | RF-024–025, RF-045–046 |
| `onMove`, `onMoveStart`, `onMoveEnd`, `onViewportChange`, `onPaneScroll`, `onPaneClick`, `onPaneContextMenu`, `onPaneMouseEnter`, `onPaneMouseMove`, `onPaneMouseLeave` | RF-029–034, RF-045–046 |
| `connectionLineType`, `connectionLineStyle`, `connectionLineComponent`, `connectionLineContainerStyle`, `connectionMode`, `connectOnClick`, `connectionRadius`, `reconnectRadius`, `isValidConnection`, `connectionDragThreshold` | RF-009–018 |
| `deleteKeyCode`, `selectionKeyCode`, `panActivationKeyCode`, `multiSelectionKeyCode`, `zoomActivationKeyCode`, `selectionOnDrag`, `selectionMode`, `selectNodesOnDrag`, `paneClickDistance`, `nodeClickDistance`, `nodeDragThreshold` | RF-024–026, RF-029, RF-051 |
| `snapToGrid`, `snapGrid`, `onlyRenderVisibleElements`, `nodesDraggable`, `nodesConnectable`, `nodesFocusable`, `nodeOrigin`, `edgesFocusable`, `edgesReconnectable`, `elementsSelectable`, `nodeExtent`, `elevateNodesOnSelect`, `elevateEdgesOnSelect`, `zIndexMode` | RF-004–009, RF-013, RF-021, RF-024–026, RF-034, RF-052 |
| `panOnDrag`, `minZoom`, `maxZoom`, `viewport`, `defaultViewport`, `translateExtent`, `preventScrolling`, `zoomOnScroll`, `zoomOnPinch`, `panOnScroll`, `panOnScrollSpeed`, `panOnScrollMode`, `zoomOnDoubleClick`, `fitView`, `fitViewOptions` | RF-029–034 |
| `autoPanOnNodeFocus`, `autoPanOnNodeDrag`, `autoPanOnConnect`, `autoPanOnSelection`, `autoPanSpeed`, `noDragClassName`, `noWheelClassName`, `noPanClassName` | RF-003, RF-025, RF-029–031, RF-052 |
| `defaultMarkerColor`, `attributionPosition`, `proOptions`, `colorMode`, `disableKeyboardA11y`, `ariaLabelConfig`, `onError`, `debug` | RF-019, RF-041, RF-049, RF-051–054 |

## Complete versioned example crosswalk

The following lists every directory inspected under `examples/react/src/examples` at the frozen source. It is an omission-review index, not an assertion that each example has passed. A named example may exercise more than the listed primary rows.

| Example directories | Primary rows |
|---|---|
| `Basic`, `Empty`, `DefaultNodes`, `DefaultNodeOverwrite`, `CustomNode`, `NodeTypeChange`, `NodeTypesObjectChange`, `UpdateNode`, `Switch`, `BrokenNodes` | RF-001–005, RF-043, RF-049 |
| `DetachedHandle`, `MovingHandles`, `Undirectional`, `EasyConnect`, `UseConnection`, `UseNodeConnections`, `Validation`, `CancelConnection`, `ReconnectEdge`, `AddNodeOnEdgeDrop` | RF-008–016, RF-055–057 |
| `Edges`, `EdgeTypes`, `EdgeRenderer`, `EdgeRouting`, `DefaultEdgeOverwrite`, `FloatingEdges`, `CustomConnectionLine`, `EdgeToolbar` | RF-017–023, RF-039, RF-056 |
| `Interaction`, `ClickDistance`, `DragHandle`, `DragNDrop`, `Intersection`, `NodeSelectionBug`, `NodeResizer`, `Subflow`, `ZIndexMode`, `Figma` | RF-003, RF-006–007, RF-024–031, RF-055, RF-060 |
| `ControlledViewport`, `Backgrounds`, `InteractiveMinimap`, `CustomMiniMapNode`, `NodeToolbar`, `ColorMode` | RF-032–041 |
| `Provider`, `MultiFlows`, `ControlledUncontrolled`, `Redux`, `Remount`, `MultiSetNodes`, `SetNodesBatching`, `Middlewares` | RF-042–046 |
| `UseReactFlow`, `UseNodesData`, `UseNodesInit`, `UseUpdateNodeInternals`, `UseOnSelectionChange`, `SaveRestore` | RF-004, RF-010, RF-043–048, RF-057 |
| `A11y`, `UseKeyPress`, `TouchDevice` | RF-029–031, RF-051–052 |
| `Hidden`, `Stress`, `Layouting`, `DevTools`, `Overview` | RF-034, RF-049–050, RF-059, RF-066 |

### Official website catalogue crosswalk

The 2026-11-09 [public catalogue](https://reactflow.dev/examples) is a separate evidence set from the 66 source directories. Every named catalogue example below remains in scope. References to Pro behavior do not claim access to its premium implementation; a source example with a similar name does not establish that behavior (notably obstacle-avoiding Edge Routing).

| Category | Named example → required rows |
|---|---|
| Overview | Feature Overview → RF-001–054 and the source `Overview` example; this is a combined demonstration of existing rows, not an additional capability. |
| Nodes | Add Node On Edge Drop → RF-055; Connection Limit → RF-014/057; Custom Nodes → RF-002/003; Delete Middle Node → RF-069; Drag Handle → RF-005/026; Easy Connect → RF-056; Intersections → RF-047/060; Node Resizer → RF-027; Node Toolbar → RF-038; Proximity Connect → RF-067; Rotatable Node → RF-061; Node Position Animation → RF-068; Stress Test → RF-050; Updating Nodes → RF-043; Shapes → RF-002/061/062. |
| Edges | Animating Edges → RF-022/063; Custom Connection Line → RF-015; Custom Edges → RF-018; Delete Edge on Drop → RF-070; Edge Label Renderer → RF-020/040; Edge Intersection → RF-071; Edge Toolbar → RF-039; Edge Types → RF-017/018; Edge Routing → RF-074; Floating Edges → RF-056; Edge Markers → RF-019; Multi Connection Line → RF-063; Reconnect Edge → RF-013; Simple Floating Edges → RF-056; Temporary Edges → RF-072; Editable Edge → RF-063. |
| Interaction | Computing Flows → RF-057; Connection Events → RF-016/045; Context Menu → RF-045/066; Contextual Zoom → RF-073; Drag and Drop → RF-055; Preventing Cycles → RF-014/057; Save and Restore → RF-048; Touch Device → RF-030; Validation → RF-014; Helper Lines → RF-061; Collaborative → RF-065; Copy and Paste → RF-058; Undo and Redo → RF-058. |
| Subflows & Grouping | Selection Grouping → RF-060; Parent Child Relation → RF-006/060; Sub Flow → RF-006/060. |
| Layout | Dagre Tree, Elkjs Tree, Elkjs Multiple Handles, Horizontal Flow, Auto Layout, Force Layout, Dynamic Layouting → RF-059; Expand and Collapse → RF-059/060; Node Collisions → RF-059/060. |
| Styling | Base Style, Dark Mode, Tailwind, Turbo Flow → RF-041/054/066: native appearance/composition capability, not a CSS runtime. |
| Whiteboard | Eraser Tool, Lasso Selection, Rectangle, Freehand Draw → RF-062. |
| Misc | Download Image, Server Side Image Creation → RF-053/064. |

Before inventory sign-off, the reviewer must reconcile both catalogues and Pro descriptions with the rows, split grouped capabilities where needed, and record unresolved source access. Do not silently remove a behavior because its premium implementation is unavailable; implement a native equivalent from an approved behavior specification and lawful references. Future documentation changes are an explicit inventory delta, not an implicit change to the frozen baseline.

## Native evidence and production gate

Use coherent demonstrator journeys to cover related IDs, existing checks and regressions for actual defects. Do not create a combinatorial test matrix for every option. Each accepted row must name the exact engine SHA, native demonstrator/consumer SHA, relevant automated checks, actual native observation, independent reviewer and disclosed limitations. A static source match, compiler, screenshot or generic egui capability does not constitute acceptance.

| Journey | Required observation |
|---|---|
| **J1 — custom interactive nodes** | Create/switch/update custom nodes with text, IME, media/button and nested scrolling controls; dynamic dimensions/ports; child actions never become board actions; multiple independent flows. |
| **J2 — complete connections** | Drag creation/reconnection, both endpoint policies, same-card distinct handles, multiple same-side named ports, validation feedback, selected/hover affordances, direct path/label actions. Cancel/stale/focus/pointer loss preserves originals; zoom/clipping/overlap and dynamic-port invalidation remain correct. |
| **J3 — geometry and navigation** | Select, marquee, move, nested groups, resize/rotate compositions, snap/extents, pan/zoom/autopan/fit and device gestures; paint/hit/focus use the same current geometry, including large/fractional coordinates. |
| **J4 — presentation and scene** | Every standard route/marker, custom edges/labels, style states, widgets/toolbars/overlays, minimap and backgrounds; demonstrate offscreen/native scene output without data mutation. |
| **J5 — event contract** | Record ordering, target identity, candidate/base, start/end/cancel and veto/defer behavior for J1–J4; callbacks do not leak duplicate or post-cancel mutations. All event-bearing rows additionally require J5. |
| **J6 — native input and accessibility** | Real OS mouse/keyboard/touch where supported, native clipboard/IME, screen reader/AccessKit actions, focus traversal/reveal and localized guidance. Report each tested OS/device; scripted RawInput is supplementary only. |
| **J7 — authority and durability** | Exact-base prepare/apply, failure/retry, cancel, Undo/Redo, copy/remap, snapshot+viewport restore and process reopen with unknown fields/large numbers/parallel edges retained. Engine acknowledgement and durable host save remain distinct. |
| **J8 — composition** | Runnable native examples for layout, dataflow, clipboard/history, grouping, drawing, collaboration and export; host adapters prove integration without pulling storage/network/media into the engine. |

Production thresholds and milestone order are specified in [PRODUCTION_PLAN.md](PRODUCTION_PLAN.md), with an independently reviewed gate: supported platforms/devices and accessibility paths; representative graph/label/port/edge densities; bounded memory, history and route/layout work; frame/input latency at measured hardware; cancellation/error recovery; API compatibility and migration policy; standalone build/package/demo reproducibility. Current defaults (12 MiB document, 5,000 nodes, 10,000 edges, integer geometry/minimum card size and zoom limits) are existing Studio-oriented constraints, not verified universal React Flow limits. Generalization must be explicit and must preserve the consumer compatibility profile.

## Architecture prerequisites and first coherent increment

1. Specify compatible named-port persistence and defaults. Preserve missing/null/explicit endpoint information, old side-only records and dangling legacy data without inventing IDs or silently reconnecting them. Keep typed validated connections separate from opaque preservation.
2. Add a generic gesture/policy layer with stable targets, exact bases and measurement epochs; explicit pointer/keyboard/wheel/touch ownership; cancelable preview state; candidate validation and ordered events. Completion emits one guarded command. Reconnect retains its original edge throughout preview.
3. Share geometry between painting, hit testing, accessible actors, labels and custom widgets. Keep relative-origin `f64` calculations and derive screen geometry only after current-frame camera changes. Expose required queries and invalidation without a competing graph.
4. Preserve the existing command/history authority while introducing generic profiles, custom node/edge surfaces, port/parent semantics and standalone consumers. Record schema/API and dependency decisions before implementation; retain upstream licensing/provenance for any reused code.

The first coherent increment is direct handle drag creation/reconnection, hover/selected handles, target feedback, original-on-cancel behavior, embedded-input ownership and direct edge/label selection. It must be accepted as an integrated workflow, then the remaining rows continue. It is **not** full React Flow parity, production readiness or full Studio Canvas acceptance.
