## ADDED Requirements

### Requirement: Visible canonical connector semantics

The adapter SHALL render stored side endpoints, source/target arrow ends and the source-compatible rounded orthogonal route using bounded relative-origin geometry. Paint, hit testing and emitted host label positions SHALL use the same current derived path. Rendering SHALL NOT modify canonical fields, omitted sides or unknown data.

#### Scenario: Select and delete a marked connector
- WHEN a user selects a rendered connector between two cards and presses Delete
- THEN the view emits one exact-base edge-only removal intent, retains both cards and does not mutate data itself
- AND the host may apply and undo through the existing model to restore exact edge identity and unknown fields

#### Scenario: Imported same-card connector
- WHEN a stored connector joins different, equal or omitted sides of the same card
- THEN a bounded visible outside-card path remains selectable and its label anchor lies outside the card
- AND no canonical geometry or side defaults are written

### Requirement: Native input ownership

Node hits SHALL take precedence over connector hits; overlapping connectors SHALL follow paint order. Host-owned controls, blocked editing and absent native focus SHALL NOT select or delete a connector. Double-clicking a connector SHALL NOT create a text card underneath it.

#### Scenario: Card covers a connection
- WHEN a user presses inside a card crossed by a connector
- THEN the card retains the pointer interaction and no hidden edge replaces its selection
