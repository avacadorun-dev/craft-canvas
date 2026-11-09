# Stable ports on the preserved document

CC-02 extends the existing canonical model; it adds no GUI gesture, dependency or application pin. `CanvasSession::prepare` remains the producer of guarded complete candidates; the host owns durable acknowledgement. The comparison is xyflow `1bc405cd113618c17fd181b931a9fe573bec9fee`, especially system `types/handles.ts`, `xyhandle/XYHandle.ts`, `xyhandle/utils.ts` and React `EdgeWrapper`.

## Version 1 schema

The optional node/edge top-level object is `craftCanvas`, recognized only when `version` is JSON unsigned integer `1`. Duplicate-key and raw-envelope preservation rules remain mandatory. Unknown members are never reconstructed through floating-point JSON values.

```json
{"craftCanvas":{"version":1,"ports":[{"id":"answer","kind":"source","side":"right","x":200,"y":30},{"id":"alternative","kind":"source","side":"right","x":200,"y":90,"start":false}]}}
```

Node ports require `id`, `kind`, `side`, `x`, `y`. Identity is node-local `(kind,id)`: `kind` is source/target, and one source plus one target may share the same textual ID. IDs are nonempty, unnormalized strings within `max_id_bytes`. Side is top/right/bottom/left and controls tangent, independently of identity. Coordinates are finite node-local f64 anchor values bounded by `max_coordinate`; fractional and outside-node anchors are allowed. Node integral/minimum-dimension rules do not apply. Translation preserves local anchors; resizing does not silently recompute them. Hosts explicitly update geometry when needed.

Optional booleans `connectable`, `start`, `end` each default true. Null is not a default. Start/end refer to the actual initiating/drop endpoint, independently of normalized source/target role. Missing ports means an empty named catalog. Unknown extension/per-port members and port-array order remain raw unless deliberately changed.

```json
{"craftCanvas":{"version":1,"fromPort":{"id":"answer","kind":"source"},"toPort":{"id":"input","kind":"target"},"reconnect":"from"}}
```

Edge fromPort/toPort are independently optional objects requiring id/kind. An absent binding uses its standard JSON Canvas node/side endpoint. Present null or malformed binding never falls back. Standard fromNode/toNode remain the only stored node references. Named edits preserve standard side fields. Optional reconnect is inherit/none/both/from/to; absence means inherit.

## Interpretation and protected mutations

`ExtensionStatus::{Absent,KnownV1,Opaque,Malformed}` distinguishes missing namespace, valid v1, unrecognized/non-object/missing-version namespace, and v1 with invalid recognized fields. Imported opaque/malformed envelopes remain preserved. Recognized capacity violations are hard `CanvasError::Limit` refusals, not malformed fallback or partially cached data. Existing structural JSON/node-reference limits remain unchanged.

Node named caches exist only for KnownV1. Legacy bindings do not inspect them: opaque/malformed node metadata does not disable a legacy endpoint. Named lookup refuses with an explicit unsupported/malformed reason. Opaque/malformed edge metadata cannot safely supply typed endpoints/reconnect policy, so those operations refuse. Unrelated text, label, marker and geometry edits remain possible.

Raw field commands obey the same protections. Before interpreting a replacement, compare namespace values: changing/removing an opaque/malformed namespace or descendant refuses; identical raw replacement is a no-op. For known/absent metadata, deliberate recognized changes must result in valid v1 or explicit namespace absence subject to final binding/removal checks; changing to an unknown version cannot bypass validation. Unknown v1 siblings may change without treating unchanged endpoints as new connections. No destructive automatic migration is introduced.

## Additive API and commands

```rust
enum PortKind { Source, Target }
struct PortKey { pub id: String, pub kind: PortKind }
struct PortDefinition {
    pub key: PortKey, pub side: Side, pub x: f64, pub y: f64,
    pub connectable: bool, pub start: bool, pub end: bool,
}
enum PortBinding { Legacy(Option<Side>), Named(PortKey) }
struct PortEndpoint { pub node_id: String, pub binding: PortBinding }
enum ReconnectPolicy { Inherit, None, Both, From, To }
enum ExtensionStatus { Absent, KnownV1, Opaque, Malformed }
```

Keep existing ConnectorEndpoint and legacy APIs. NodeRecord exposes `ports() -> &[PortDefinition]` (empty unless KnownV1), indexed `port(&PortKey) -> Option<&PortDefinition>`, and `port_extension_status() -> ExtensionStatus`. EdgeRecord exposes the same status method plus `endpoint(ConnectorEndpointKind) -> Result<PortEndpoint>`, `reconnect_policy() -> Result<ReconnectPolicy>`, and constructor `connect_ports(id, from, to, limits) -> Result<EdgeRecord>`. Endpoint/policy reads refuse opaque/malformed interpretation; construction alone grants no connection acceptance. `PortDefinition::world_anchor(WorldRect) -> Result<WorldPoint>` resolves its local anchor against current node geometry and refuses non-finite results.

Internal helpers: `NodeRecord::with_ports(&[PortDefinition], &CanvasLimits)`, `EdgeRecord::with_port_endpoint(ConnectorEndpointKind, &PortEndpoint, &CanvasLimits)`, `EdgeRecord::with_reconnect_policy(ReconnectPolicy, &CanvasLimits)`. Each returns a complete record. Port replacement merges known fields into retained `(kind,id)` raw objects, retaining their unknown members; deleted IDs are deliberate removal. Unchanged decoded fields retain raw omitted/default/numeric representation. Changing only an endpoint node retains its unchanged raw side (including missing/null/empty). A semantic no-op retains complete raw bytes.

```rust
SetPorts { node_id: String, ports: Vec<PortDefinition> }
SetReconnectPolicy { edge_id: String, policy: ReconnectPolicy }
ConnectPorts { edge: EdgeRecord, index: usize, start: ConnectorEndpointKind }
ReconnectPort {
    edge_id: String, endpoint: ConnectorEndpointKind,
    expected: PortEndpoint, replacement: PortEndpoint,
}
```

Exact transaction base guards all commands; reconnect also compares expected endpoint. Legacy ReconnectEdge refuses to ignore/overwrite a named binding. InsertEdge deterministically starts From; ConnectPorts carries actual start. A one-end reconnect starts at the unchanged opposite end. Raw two-end replacement starts From. Pure validation rejects contradictory reconnect initiators with InvalidInitiator, so preview cannot bypass final start/end availability. #3 must retain the original binding when a drop returns to the unchanged displayed fallback, rather than materializing a side.

## Pure validation and original policy

```rust
enum ConnectionMode { Strict, Loose } // default Strict
enum ConnectionOperation {
    Create { edge_id: String },
    Reconnect { edge_id: String, endpoint: ConnectorEndpointKind },
    ReconnectBoth { edge_id: String },
}
struct ConnectionRequest {
    pub from: PortEndpoint, pub to: PortEndpoint,
    pub start: ConnectorEndpointKind,
    pub operation: ConnectionOperation,
}
fn validate_connection(
    document: &CanvasDocument, request: &ConnectionRequest,
    dialect: &dyn CanvasDialect,
) -> std::result::Result<(), ConnectionError>;
```

ConnectionError distinguishes missing node/port/edge, opaque/malformed metadata, unavailable node/port/start/end, direction mismatch, same-port loose connection, disabled reconnect, changed stationary endpoint, invalid initiator/inherited host default and host rejection. CanvasError wraps failed validation. Create carries the allocated edge ID so a host can exclude the proposed edge consistently during preview and final-candidate checks. No validation grants save authority or performs I/O/mutation/ID allocation.

Add defaulted CanvasDialect methods `connection_mode() -> ConnectionMode` (Strict), `default_reconnect_policy() -> ReconnectPolicy` (Both), and `validate_connection(&CanvasDocument, &ConnectionRequest) -> Result<(), ConnectionError>` (Ok). A host default of Inherit refuses with InvalidReconnectDefault, rather than recursively inheriting. Existing can_connect remains mandatory for deliberate new/reconnected connections. The host callback receives the actual operation after structural/policy checks.

Requests use normalized stored from/to roles. Target-start supplies source as from, starting target as to, and start To. Strict requires Source at from and Target at to; legacy kind is inferred from normalized role. Strict allows same-node opposite-kind handles sharing a textual ID, matching pinned upstream. Loose allows either kind but refuses same-node named handles sharing textual ID, regardless of kind. Loose legacy identity compares exact Legacy(side), without inventing a rendered fallback; legacy and named identities are distinct namespaces. Default Strict retains legacy loops. Same-node distinct ports and parallel edge IDs remain allowed. Lookup never picks the first named port.

Both endpoints must resolve and satisfy node/port eligibility. Start/end flags follow the actual initiating/drop ends. Saved edges remain when availability is later disabled. Internal validation takes original and candidate documents, or explicit original edge: resolve endpoints against final candidate; authorize reconnect against original edge override and original host context. One-end reconnect checks unchanged stationary endpoint; ReconnectBoth authorizes both ends without that constraint. Changing policy in the same transaction cannot authorize previously forbidden reconnection. Public validation uses one document for both roles.

## Dynamic catalogs, final-state comparison and copying

Classify final semantic changes across typed/raw command paths. New authored edges and changed endpoint bindings receive complete validation. Label/marker-only edits do not. Two-end raw changes are ReconnectBoth, not Create. Index incident edges once when catalogs change. Removal/rename/kind change of a previously resolved named port refuses if its final edge remains broken. The same transaction may reconnect/delete affected edges or restore the port in any command order. Geometry/availability updates preserve identity and saved edges.

Imported unresolved named references remain exact under unrelated edits; new/reconnected endpoints must resolve. There is no silent reassignment. Existing missing-node import rejection remains. Node deletion retains existing incident-edge deletion semantics.

Duplicate keeps node-local port IDs and remaps only known standard node/edge IDs and standard edge node references. Opaque bytes and unresolved copied bindings remain preserved, without guessing unknown embedded references. A canonical clone is a preservation copy, not new connection authoring: retain existing clone eligibility checks but do not refuse solely for unavailable named metadata. Fresh InsertEdge/ConnectPorts receives no such exemption.

## Bounds and verification

Parse definitions and `(kind,id)` index once per record. Add positive `CanvasLimits.max_ports_per_node`, default 256. Check count before allocating the typed cache; exceeding a recognized count bound is a hard refusal. Total cached count is bounded by node count times per-node count and snapshot bytes, using checked arithmetic. Reuse ID/coordinate bounds; computed world anchors must be finite. Account parsed ID/index payload honestly alongside raw bytes, without pretending container overhead is zero.

Charge catalog/index construction, incident-edge classification and validation through WorkControl; avoid scanning every port for every edge. Host policy and geometry caches can change without ContentIdentity: freeze/recheck host context before durable acknowledgement; #3 explicitly invalidates gesture measurements. Preview/model tests do not establish native acceptance.

Hosted evidence covers same-side identities, strict/loose target-start, endpoint policy, opaque/malformed preservation, raw setter equivalence, legacy no-op lexemes, atomic removal repair, duplication, stale rejection, bounds, Undo and semantic round trips. Independent public API review is required before closing #2.
