## ADDED Requirements

### Requirement: Preserve bounded canonical boards

The engine SHALL preserve exact no-edit bytes and unrelated raw record/top-level JSON through known edits, including arbitrary-precision unknown numbers and array order. It SHALL reject duplicate keys, excessive nesting, invalid geometry, unknown endpoints and capacity overflow before mutation.

#### Scenario: Edit a known field in a future envelope
- **WHEN** a valid board contains unknown nested fields, unusual numeric lexemes and ordered records and an author changes one known text/geometry field
- **THEN** the intended field changes, unknown values and unedited records remain preserved, and Undo restores the complete prior board

### Requirement: Exact-base atomic changes

Prepare SHALL validate all commands at one exact private session identity without mutation. Apply, Undo and Redo SHALL refuse stale or foreign identities and retain complete reversible operations within explicit budgets.

#### Scenario: A host receives a late prepared result
- **WHEN** the session changes before that result is applied
- **THEN** Apply refuses and the newer board and history remain unchanged

### Requirement: Precision-safe native intents

The native adapter SHALL use relative-origin f64 camera math and emit one semantic transaction per completed gesture. Escape, focus loss and base changes SHALL cancel transient work. Embedded host controls and active text composition SHALL not trigger board shortcuts.

#### Scenario: Move a card near a world extreme
- **WHEN** a card near ±10,000,000 is dragged at zoom 0.12 or 3.0
- **THEN** hit testing and preview retain local precision and release emits one guarded bounded geometry operation

### Requirement: Host capability boundary

File, URL and extension resource strings SHALL remain inert. The host SHALL own media, clipboard, labels, resource authorization, durable save and assistant review.

#### Scenario: Open an imported file node
- **WHEN** a generic board contains a file or URL reference
- **THEN** generic parsing and rendering perform no filesystem/network request and navigation is an explicit host intent

### Requirement: Source-compatible history capacity

Past/future history SHALL retain bounded exact raw snapshots without retaining parsed historical indexes. The current source SHALL have a separate12MiB bound from the32MiB historical source budget. Complete operation metadata and current parsed payload SHALL be reported separately; source-byte limits SHALL NOT be represented as allocator or RSS measurements.

#### Scenario: Parsed caches are larger than encoded snapshots
- **WHEN** a source-compatible board is edited while its two retained encoded snapshots fit the history budget
- **THEN** both complete Undo steps remain available, current parsed-cache duplication does not consume their source slots, and Undo/Redo preserve the acknowledged historical budget and exact JSON
