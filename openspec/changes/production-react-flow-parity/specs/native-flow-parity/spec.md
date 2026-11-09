## ADDED Requirements

### Requirement: Complete native capability coverage

CraftCanvas SHALL provide native functional equivalents for every required RF inventory capability. Core and documented composition provenance SHALL remain explicit. Inventory omissions SHALL be independently reviewed before parity acceptance; source presence alone SHALL NOT establish working behavior.

#### Scenario: Qualify a capability
- WHEN a capability is marked accepted
- THEN its exact implementation, reachable standalone API/example, relevant checks and independent native evidence are recorded

#### Scenario: Classify browser and example behavior
- WHEN a browser-specific mechanism or documented example has no direct Rust API counterpart
- THEN its observable behavior remains mapped to a native equivalent or an explicit unfinished requirement, not silently removed

### Requirement: Preserve canonical authority and compatibility

Generic extensions SHALL retain exact no-edit bytes, unknown fields, stable identities, original array order and guarded history. Named ports and parent/custom data SHALL have explicit compatible defaults. Presentation and input previews SHALL NOT mutate canonical state or imply durable acknowledgement.

#### Scenario: Extend a legacy edge
- WHEN a legacy edge with omitted side fields participates in a named-port-capable view
- THEN rendering defaults do not rewrite its raw endpoint and an unchanged reconnect produces no mutation

#### Scenario: Refuse unsafe state
- WHEN a command is stale, cancelled, invalid or requires an unsupported extension mutation
- THEN no partial canonical change occurs and existing host recovery data remains available

### Requirement: Controlled native interaction

Native graph gestures SHALL have stable targets, explicit input ownership and observable start/update/end/cancel semantics separate from commit authority. Creation and reconnection SHALL support direct dragging with valid-target feedback. A completed edit SHALL emit one guarded action; cancellation SHALL preserve the original graph.

#### Scenario: Reconnect with an embedded editor
- WHEN a connection gesture competes with a focused native editor, media control or overlay
- THEN the rightful input owner consumes the sequence and a consumed release cannot trigger another board command

#### Scenario: End without a mutation
- WHEN a gesture is cancelled, invalid or unchanged
- THEN its terminal lifecycle is observable without inventing a changed transaction

### Requirement: Sequential qualified delivery

Library implementation SHALL follow the ordered roadmap with one active issue. Each closure SHALL record exact source, relevant checks, required native observations, independent review and limitations. WriterCraft integration SHALL adopt only an exact qualified engine handoff; standalone qualification SHALL NOT accept application workflows.

#### Scenario: Reach the consumer checkpoint
- WHEN CC-12 is accepted
- THEN every current consumer-required engine primitive is qualified and the app receives migration instructions, while unfinished full React Flow requirements remain open

### Requirement: Honest production and package readiness

Production acceptance SHALL require complete capability coverage, preserved-data safety, supported native input/accessibility, measured performance within frozen applicable targets and verified standalone packages/docs. Published versions SHALL identify their precise supported scope. Unavailable platform evidence SHALL remain pending.

#### Scenario: Publish an engineering package
- WHEN a partial but qualified prerelease is published
- THEN its exact source, checks, notices and limitations are documented without claiming full parity or production readiness

#### Scenario: Fail a capacity target
- WHEN a supported-capacity workload misses its frozen performance or reliability requirement
- THEN the relevant issue remains open and the source-valid capacity is not silently reduced
