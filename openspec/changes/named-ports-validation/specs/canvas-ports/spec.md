## ADDED Requirements

### Requirement: Stable named port identity

Connections SHALL distinguish node-local `(kind,id)` identities from sides and finite node-local f64 anchors. The optional `craftCanvas` v1 codec SHALL use the exact schema/defaults in design.md. Multiple ports per side and source/target ports sharing textual IDs SHALL remain distinct. Unknown raw members and legacy representations SHALL remain preserved.

#### Scenario: Multiple ports on one side
- WHEN a node exposes two different named ports on the same side
- THEN connections retain their distinct port bindings through serialization, duplication and Undo/Redo

### Requirement: Uniform final endpoint validation

Every newly created or deliberately changed endpoint SHALL receive final connection validation against the complete prepared candidate, regardless of command entry point. Preview validation SHALL NOT grant Apply authority. Unrelated label or marker edits SHALL preserve existing endpoint compatibility.

#### Scenario: Raw field command cannot bypass endpoint policy
- WHEN a field update changes an edge endpoint to a node that the host refuses to connect
- THEN preparation refuses without modifying canonical data or history, just as an explicit reconnect would

#### Scenario: Unrelated edit on a preserved edge
- WHEN an edge label or marker changes without changing its endpoint
- THEN endpoint eligibility alone does not reject the unrelated change or rewrite the preserved endpoint

### Requirement: Normalized connection requests

Requests SHALL carry normalized from/to endpoints, actual start end and Create/Reconnect/ReconnectBoth operation. Create SHALL carry the allocated edge ID for consistent host checks. Node/port availability and start/end flags SHALL follow the actual initiating/drop end.

#### Scenario: Target-start normalization
- WHEN a connection starts at a target and ends at a source
- THEN the request stores source as from, target as to and start To, checking target start and source end availability

### Requirement: Strict and loose direction rules

Strict SHALL default to source-to-target roles and allow opposite-kind same-node handles sharing a textual ID. Loose SHALL allow same kinds but reject same-node named handles sharing textual ID. Legacy kinds SHALL derive from edge role; loose legacy identity SHALL use the exact Legacy(side) binding. Parallel edges SHALL remain permitted.

#### Scenario: Same-node handles sharing a textual ID
- WHEN one node has source and target ports with the same textual ID
- THEN Strict may connect them while Loose refuses that same-node textual identity, subject to the remaining eligibility rules

### Requirement: Deterministic reconnect initiators

A one-end reconnect SHALL start at its unchanged opposite endpoint; ReconnectBoth SHALL start From. Contradictory supplied initiators SHALL refuse in pure validation as well as final preparation.

#### Scenario: Contradictory reconnect initiator
- WHEN a one-end reconnect request claims its changed endpoint as the initiator
- THEN both preview and final validation refuse instead of applying different start/end availability rules

### Requirement: Compatible opaque extensions

Typed port operations SHALL have explicit schema/version defaults and bounded validation. Unsupported or unresolved imported port data SHALL remain preserved without invented fallback bindings. Unchanged operations SHALL retain raw missing/null/explicit data; unsupported deliberate mutations SHALL refuse before changing data.

#### Scenario: Future extension is preserved
- WHEN a board contains an unsupported port extension and receives an unrelated supported edit
- THEN the opaque extension remains unchanged, and a typed mutation of that extension refuses actionably

#### Scenario: Raw legacy side survives node-only reconnect
- WHEN only a legacy endpoint node changes
- THEN an unchanged side retains its original omitted, null, empty or explicit representation

### Requirement: Explicit extension interpretation

The codec SHALL distinguish Absent/KnownV1/Opaque/Malformed. An absent binding SHALL mean legacy; malformed present bindings SHALL never fall back. Opaque/malformed node port metadata SHALL NOT disable a legacy binding that does not interpret it.

#### Scenario: Legacy binding on a node with opaque ports
- WHEN a legacy endpoint uses a node with opaque named-port metadata
- THEN eligibility does not require interpreting that catalog, while a malformed present edge binding receives no legacy fallback

### Requirement: Uniform raw extension protection

Raw namespace/descendant mutations SHALL receive the same protection as typed mutation.

#### Scenario: Raw mutation cannot overwrite unsupported metadata
- WHEN a generic setter changes/removes an opaque or malformed namespace or descendant
- THEN preparation refuses before mutation, while identical raw replacement remains a no-op

### Requirement: Explicit dynamic and reconnect policy

The model SHALL expose pure candidate validation with actionable outcomes and per-end reconnection policy. Removed or unavailable ports SHALL have explicit retained-data behavior, with no silent reassignment. Final validation SHALL recheck exact state and policies before a complete atomic action is accepted.

#### Scenario: Port changes between preview and commit
- WHEN a previously valid port becomes unavailable or its exact candidate base changes before preparation
- THEN the operation refuses without partial mutation, while existing saved edges remain preserved according to the explicit removal policy

### Requirement: Original-edge reconnect authorization

Reconnect policy SHALL be inherit/none/both/from/to, absent inherit, default host Both; an inherited host default SHALL refuse. Authorization SHALL read the original edge policy, while endpoint resolution SHALL read the final candidate. A one-end reconnect SHALL preserve its stationary endpoint. Raw two-end mutation SHALL be ReconnectBoth and authorize both ends. Policy changes in the same transaction SHALL NOT authorize their own bypass.

#### Scenario: Original policy prevents bypass
- WHEN a transaction enables reconnect on a previously disabled edge and reconnects it
- THEN original policy refuses the reconnect without partial data or history changes

### Requirement: Final-state catalog repair

Removal/rename/kind changes SHALL refuse if a previously resolved binding becomes broken in the final candidate. Atomic edge repair/deletion SHALL be allowed regardless of command order. Geometry/availability changes SHALL preserve existing saved edges. Imported unresolved bindings SHALL survive unrelated edits; newly authored/reconnected bindings SHALL resolve. Existing missing-node import rejection SHALL remain unchanged.

#### Scenario: Final-state removal repair
- WHEN a transaction removes a port and reconnects or deletes every affected resolved edge
- THEN final validation may accept in either command order, with no intermediate canonical mutation

### Requirement: Preservation-aware copying

Duplicate SHALL preserve node-local port IDs, unknown raw metadata and unresolved saved bindings while remapping known standard node/edge identities. It SHALL NOT invent mappings for opaque references or treat preservation copies as new authoring; existing clone eligibility remains mandatory.

#### Scenario: Copy unresolved saved bindings
- WHEN an existing selection with unresolved named metadata is duplicated
- THEN known node references are remapped while port IDs/unknown fields remain preserved without automatic repair

### Requirement: Bounded port lookup

Named lookup SHALL use a per-record index with configured positive port-count limits, bounded IDs/geometry, charged incident-edge work and honest cache-payload accounting. Recognized capacity violations SHALL refuse rather than produce a partial or opaque cache.

#### Scenario: Capacity refuses before mutation
- WHEN recognized ports exceed configured count or geometry limits
- THEN interpretation/preparation refuses with an actionable limit error and no partial cache or canonical change

### Requirement: Scoped qualification

CC-02 SHALL record exact-source hosted checks, focused codec/semantic/Undo cases and independent public API review. It SHALL NOT claim native gesture acceptance or repin the application. CC-03 SHALL wait for complete CC-02 acceptance.

#### Scenario: Accept the model milestone
- WHEN CC-02 is offered for completion
- THEN its exact-source hosted checks and independent API review must establish its full scope before CC-03 starts, while native gesture acceptance and the application pin remain outside that acceptance
