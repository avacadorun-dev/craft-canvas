## ADDED Requirements

### Requirement: Direct handle authoring

The native view SHALL create connections by dragging visible named or legacy ports and reconnect either end of a selected edge. Target feedback and routed preview SHALL use the exact painted frame geometry and CC-02 validation, including actual initiation, same-node distinct ports and parallel edges.

#### Scenario: Target-start connection
- WHEN a target port starts a drag onto a valid source port
- THEN the stored source/target roles are normalized, actual initiation is retained and one guarded creation intent is emitted on valid release

### Requirement: Shared frame geometry

Port paint, edge routing, hit tests and observations SHALL share one camera and current-frame port geometry. Measured named anchors SHALL retain canonical identity and support fractional/outside-card positions. Unresolved metadata SHALL NOT gain a fabricated legacy fallback.

#### Scenario: Pan and zoom near a named port
- WHEN a connection begins after camera navigation
- THEN painted and hit positions agree and the exact visible named identity is selected

### Requirement: Preserved reconnection and no-op

Reconnection SHALL retain the original canonical edge until host acknowledgement. Dropping onto its unchanged displayed endpoint SHALL restore the original raw binding before comparison, emitting no transaction or history step when unchanged.

#### Scenario: Omitted legacy side
- WHEN an edge with an omitted side is reconnected to its original displayed Top anchor
- THEN its exact bytes remain unchanged and an explicit no-op end observation is emitted

### Requirement: Complete gesture lifecycle

Connection Begin/Update/End/Cancel observations SHALL be independent of commands. Every begun gesture SHALL have one terminal outcome. Escape, focus/pointer loss, outside release, context/generation changes and host takeover SHALL terminate capture while retaining ownership for the terminal frame.

#### Scenario: Interrupted release
- WHEN a connection is cancelled and a release occurs in the same frame
- THEN no edge mutation, selection, movement or text-opening action reuses the terminal input

### Requirement: Ordered native ownership

Embedded pointer, keyboard/IME, scroll/pinch, touch and overlay ownership SHALL precede board gestures. Input SHALL preserve event order and initiating touch identity; synthesized pointer events SHALL NOT duplicate a touch completion.

#### Scenario: Touch release followed by pointer loss
- WHEN the initiating touch releases validly and PointerGone follows in the same event batch
- THEN the release completes once, while PointerGone preceding release cancels

### Requirement: Exact retained acknowledgement

A successful release SHALL emit one exact-base transaction and retain pending presentation without automatic resubmission. Binding a prepared candidate and acknowledgement SHALL check exact base, next revision and full candidate bytes. Rejection SHALL retain a bounded retryable request; unrelated changes SHALL surface a conflict.

#### Scenario: Delayed or rejected save
- WHEN a host delays or rejects a submitted connection
- THEN the canonical edge stays unchanged, later frames emit no duplicate and explicit retry remains guarded against stale state

### Requirement: Standalone native acceptance

CC-03 SHALL pass hosted exact-source checks and independent compiled interaction review through an isolated native showcase. Evidence SHALL distinguish actual rendered readback, injected native events, OS interaction and physical touchscreen observations. Existing application previews and pins SHALL remain untouched.

#### Scenario: Four coherent acceptance journeys
- WHEN the qualified standalone candidate is reviewed
- THEN connection creation, endpoint revision/no-op, interruption/recovery and embedded-control journeys have exact source, captures, interaction and data/history evidence

#### Scenario: Ordinary direct manipulation after CV-03
- WHEN the ordinary showcase starts with immediate acknowledgement and Testing collapsed
- THEN the reviewer can create a connection, move a card through its unobstructed header, select/delete an edge and Undo without navigating the failure-injection controls
- AND a held/conflicting request remains visibly recoverable outside Testing, with resumed direct editing after explicit safe discard
- AND embedded text/control input does not become graph movement or deletion

#### Scenario: Coalesced ordinary drag
- WHEN a native frame contains a primary press on an unobstructed card followed by movement and release
- THEN the geometry gesture uses the actual press position and emits one guarded movement transaction on release
- AND outside-origin, embedded-control, consumed connection and interrupted input cannot start a competing geometry gesture
