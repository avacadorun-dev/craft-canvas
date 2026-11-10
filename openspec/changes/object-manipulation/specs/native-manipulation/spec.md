## ADDED Requirements

### Requirement: Ownership terminates transient geometry

An active move, resize or marquee SHALL end without a transaction when an embedded control or explicit host ownership takes over, or its accepted pointer path enters a current input exclusion. It SHALL NOT merely pause and resume after the owner/exclusion disappears. The whole accepted primary cycle SHALL use ordered press/move/release positions; later hover after a completed release SHALL NOT retroactively cancel the action. An exclusion appearing under a stationary captured pointer SHALL cancel that capture. No document/history change may occur before the host applies a guarded transaction.

#### Scenario: Release over an overlay then move elsewhere
- WHEN a move or resize is active and its release is covered by a current host exclusion
- THEN the gesture ends without a transaction and canonical data/history stay unchanged
- AND removing the overlay or moving the pointer cannot revive or commit the old gesture
- AND a new clean primary drag can start normally

#### Scenario: Global ownership takeover
- WHEN the host takes pointer, keyboard/IME, scroll or touch ownership during transient geometry
- THEN capture terminates without a transaction even if legacy body ownership flags are false
- AND a later unowned release cannot submit the cancelled change

#### Scenario: Ordered exclusion and release
- WHEN the accepted drag path enters an exclusion before release in one event batch
- THEN it cancels even if later pointer coordinates leave that exclusion
- WHEN release completes outside exclusions and later hover enters an exclusion in the same batch
- THEN that later hover cannot invalidate the completed action

### Requirement: Complete compatible manipulation

The completed CC-04 issue SHALL expose configurable selection, geometry/resize/aspect/rotation and parent/group policies with explicit lifecycle observations and a single exact-base completed intent. Paint, hits and preview SHALL share the same current transform. Pointer, keyboard and headless commands SHALL enforce the same canonical edit policies. Persistent parent cycles SHALL refuse before mutation; spatial grouping and presentation frames SHALL retain their distinct source semantics.

#### Scenario: Size an image without changing its asset
- WHEN an authorized host resizes a synthetic image card under the selected aspect/minimum policy
- THEN only its allowed display geometry changes, original asset references and unknown data remain intact
- AND cancel has no effect and Undo/Redo restore the exact accepted geometry history

### Requirement: Host-mediated placement

The completed CC-04 issue SHALL provide screen/world placement and bounded drop preview/insertion primitives for authorized external/project payloads. Payload acquisition, resource authorization, byte ingestion and ID allocation SHALL remain host responsibilities. Invalid/stale/cancelled placement SHALL produce no partial graph mutation.

### Requirement: Scoped acceptance

CC-04 SHALL retain exact-source hosted qualification and independent compiled manipulation evidence covering overlap/zoom/clipping, cancellation, history and source-valid large coordinates. Partial correction/model-only increments SHALL remain explicitly partial until the full issue and source-backed consumer requirements pass. The existing user-controlled preview SHALL remain intact.

### Requirement: Independent selection and visibility policies

Per-node and per-edge selection, deletion and visibility SHALL have explicit compatible defaults through the host dialect. Node wrapper focusability SHALL be independent of embedded body focus and selectability. Hidden nodes and their incident edges SHALL contribute no native bodies/ports/hits; canonical data SHALL remain unchanged. A visible nonselectable node SHALL remain an input blocker and SHALL be independently movable when its move policy permits, without changing another selection. Marquee SHALL support intersection and complete containment, and the view SHALL report changed filtered stable selections without repeated idle events.

#### Scenario: Select visible editable targets
- WHEN a board contains visible nonselectable nodes, hidden nodes, protected nodes and incident edges
- THEN click, additive, marquee, focus-driven and Select All behavior use the same explicit policy
- AND hiding or disabling selection removes stale retained selection without deleting records
- AND embedded editor focus remains independent from wrapper focusability

### Requirement: Protected removal is atomic

Forward removal SHALL validate every explicit target and current incident-edge removal against both the current candidate and original canonical record policies before mutation. A refusal SHALL preserve the complete session, history and raw data. A field edit earlier in the same transaction SHALL NOT bypass original deletion protection. Hidden/selectable flags SHALL NOT substitute for deletion authorization.

#### Scenario: Protected incident relation
- WHEN a selected node has a protected edge that would be removed incidentally
- THEN the entire removal refuses, including other selected nodes and preceding edits
- AND no dangling edge or partial deletion is published

#### Scenario: Group membership and exact history
- WHEN an unprotected spatial group is removed
- THEN enclosed protected members remain in their original order with exact data
- AND Undo/Redo retain the existing exact historical restore contract
