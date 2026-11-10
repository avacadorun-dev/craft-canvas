# CC-04 increment 2 — selection and deletion policy

Status: independently reviewed and approved for bounded implementation on2026-11-10. Reuse the qualified fa4cf13 input path. This increment completes independent selection/visibility/wrapper-focus flags and forward removal policy, plus marquee inclusion and selection-change observation. Full geometry lifecycle/thresholds, transformed stacking, sizing/parents and drop insertion remain mandatory following increments within #4.

## Compatible model API

Add two Copy/Debug/PartialEq/Eq policy structs with explicit Default implementations:

- NodeInteractionPolicy { selectable: bool, deletable: bool, focusable: bool, hidden: bool }.
- EdgeInteractionPolicy { selectable: bool, deletable: bool, hidden: bool }.

Defaults are true for selectable/deletable/focusable and false for hidden, including preserved unknown node kinds. Add defaulted CanvasDialect::node_interaction(&NodeRecord) and edge_interaction(&EdgeRecord). Keep the existing NodePolicy fields/literals and movable/resizable/group semantics unchanged. Node focusability governs the board wrapper only; no unsupported edge-focus flag is exported before its native focus surface exists. No stored field, including a string named locked or hidden, gains automatic meaning. The host dialect explicitly maps its own data/policy.

Selectability does not imply movability, and hidden is presentation rather than deletion authorization. A visible nonselectable node remains the topmost input blocker and may be dragged independently when its existing movable policy allows it, without replacing the current selection. Its sole explicit drag root is the clicked node; normal spatial-group expansion still applies, so an enclosed selected member may move because of membership, not because the existing selection was used as drag roots. Unknown node geometry restrictions remain unchanged. Headless queries continue to expose canonical records, including hidden records.

## Atomic removal policy

For RemoveSelection, first validate every explicit ID and resolve the current explicit/incident removal closure. Before changing retained arrays or mappings, check deletable for each removed current record AND its original canonical record with the same ID, if present. This prevents earlier SetNodeFields/SetEdgeFields commands from clearing protection and deleting a record in one transaction. Newly inserted protected records are checked in their current form. A newly protected record also refuses. Policy changes require a separate accepted command before deletion under the new policy.

Any protected explicit node/edge or protected current incident edge refuses the complete transaction with NotEditable(record ID). Preserve existing NotFound behavior. Do not silently filter a mixed selection into partial deletion; do not keep dangling protected edges. Group removal continues to remove only the selected group, without expanding enclosed members. An edge explicitly reconnected away earlier in the transaction survives and is not part of the removal closure. Charge validation traversals with the existing WorkControl; retain exact raw envelopes, array order, mapping and connection bookkeeping.

These flags define forward interaction/removal policy, not resource authorization. Existing prepare_undo/redo and prepared Apply retain their exact history/identity contract and do not acquire a new mutable-dialect recheck. Hosts continue to serialize external policy changes and authorize persistence; this increment does not claim deferred-delete middleware (RF-028 broader CC-13).

## One derived visibility and selection rule

Every show prunes absent, hidden and nonselectable IDs from the retained selection, including externally supplied sets and unchanged-session policy updates. An edge is visibly present only if its own hidden flag is false and both endpoint nodes are present and not hidden. Its selectable flag governs user/retained selection independently. A hidden node contributes no body, wrapper, port, hit or occlusion entry; its incident edges contribute no paint/hit/reconnect handle. Raw records and headless geometry stay unchanged. Edge nonselectability must block selecting a lower overlapping edge through its visible route; a visible nonselectable node likewise remains topmost. Keep canonical ordering for paint/hits; custom z/parent/elevation remains subsequent work.

Use the same filtered nodes for click/additive selection, marquee and Select All. Use the same visible/selectable edge rule for path selection and Select All. Focus gained on a nonselectable wrapper never selects it. Disable only the wrapper's FOCUSABLE sense (CLICK/DRAG remain independently usable); surrender only that exact wrapper ID if its policy becomes unfocusable. Embedded TextEdit/media focus and events remain host-owned and unchanged.

A policy-filtered selection change cancels conflicting geometry before further manipulation. Independently validate captured geometry roots/members each frame: cancel an affected unsubmitted move/resize if a captured record disappears, becomes hidden compared with its captured visibility, or loses its relevant move/resize permission. A directly dragged nonselectable node is covered even when the retained selection does not change. Existing already-hidden spatial members keep their normal group membership; hiding is not a general headless movement prohibition. Capture the bounded initial hidden-member set where necessary to distinguish that case.

Current visibility also invalidates unsubmitted connection capture independently of selection, including an unselected origin/target and the original reconnect edge. Use the existing GeometryUnavailable cancellation reason and terminal ownership; no new request may emerge from that cancelled frame. An existing submitted connection retains its exact request, receipt, recovery state and acknowledgement authority. Suppress only its preview presentation while either endpoint or its original reconnect edge is hidden; if visibility returns, the same retained pending preview may return. Offscreen culling is not hiding. Do not reinterpret a visibility change as save failure or discard. Native Delete/Backspace emits the complete retained selection; final model refusal is authoritative and actionable. Cut stays a host intent with the same eventual removal policy, without clipboard I/O in the library.

## Marquee and observable selection

Add SelectionMode { Intersect, Contain }, default Intersect, and CanvasViewConfig::selection_mode. Contain requires the entire canonical axis-aligned node bounds inside the marquee; Intersect preserves current inclusive intersection behavior. This affects marquee inclusion only, not geometry validation or command semantics. Existing additive/toggle modifiers remain compatible.

Add SelectionChange { base: ContentIdentity, selection: Selection } and CanvasViewOutput::selection_change: Option<SelectionChange>. Treat the initial empty set as already reported. Emit the final filtered stable set when it differs from the last reported selection; unchanged frames emit None, including a mere revision change with the same set. Programmatic set_selection is reported after filtering on the next show. Retain at most one bounded previous reported set, and clone a snapshot only when changed. This is a presentation observation, not mutation/save authority. Full marquee Begin/End/Cancel, selection rollback and geometry lifecycle remain explicitly following work; this event does not pretend to provide them.

## Verification and ownership

Model owner: crates/craft-canvas-model/src/lib.rs, commands.rs and one focused interaction-policy test file. Adapter owner: crates/craft-canvas-egui/src/view.rs, port_geometry.rs, connection.rs (bounded visibility lifecycle only), lib.rs, and one focused selection-policy event test file. Root owns docs, integration, commits/push and hosted dispatch; independent reviewer reviews the same frozen increment. No lock, dependency, host application, current preview or original Studio changes.

Use existing147 checks plus focused semantic regressions: atomic protected-node/incident-edge refusal including field-edit bypass, compatible successful delete/Undo with unknown records and group-member preservation; hidden paint/ports/route/hits consistency including stationary active capture and retained-but-unpainted pending acknowledgement; nonselectable independent dragging and focus safety; marquee modes and Select All; selection event no-duplicates. Avoid a property-combination matrix. Exact hosted fmt/tests/Clippy and representative compiled full-issue input acceptance remain required; source checks alone cannot close #4.
