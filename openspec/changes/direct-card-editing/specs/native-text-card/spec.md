## ADDED Requirements

### Requirement: DCE-1 direct card input

The adapter SHALL provide a real multiline native input inside the reserved card body at the actual zoom and clipping geometry. Text selection, clipboard, local Undo and IME SHALL retain child input ownership instead of becoming board commands.

#### Scenario: Text card is edited
- **WHEN** the host activates an existing or newly created text card
- **THEN** the native editor SHALL request focus in that card and retain its local draft through ordinary OS focus changes
- **AND** Done/Cancel SHALL not be emitted from IME composition key events

### Requirement: DCE-2 exact acknowledgement preserves later text

A submitted draft SHALL remain separate from later native input. Acknowledgement MUST verify the same canonical session and exact submitted node source, then rebase without replacing later text or resetting the widget identity.

#### Scenario: Typing continues during persistence
- **WHEN** a submitted edit is saved/applied while more local text has been entered
- **THEN** acknowledgement SHALL retain that later text as a dirty draft against the new exact base

#### Scenario: Wrong source or foreign session is acknowledged
- **WHEN** an acknowledgement does not contain the exact submitted source in the same session
- **THEN** it SHALL be rejected and the complete local draft/submission SHALL remain available

### Requirement: DCE-3 new-card input before insertion acknowledgement

The same native card editor SHALL accept input on its first mounted frame while the host persists the original insertion. The provisional record SHALL be rendered only at its exact canonical base, without a shadow session or canonical mutation. The creating pointer action SHALL not be interpreted as finishing the new edit.

#### Scenario: Typing precedes the insertion acknowledgement
- **WHEN** the first mounted frame already contains native text or IME input
- **THEN** the in-card widget SHALL own and process that input without an empty bootstrap frame
- **AND** the host's exact next-revision, byte-identical insertion acknowledgement SHALL retain the widget, text, selection, composition and local Undo
- **AND** a foreign, stale or different-record acknowledgement SHALL preserve the pending draft without promoting it

#### Scenario: Insertion is delayed or fails
- **WHEN** no successful insertion acknowledgement has arrived
- **THEN** the draft SHALL remain editable and ordinary text submission SHALL be refused
- **AND** Escape SHALL report a cancel intent without destroying the draft or emitting a board mutation; the host SHALL resolve any in-flight insertion before discarding it
- **AND** deliberate preservation MAY prepare one host-owned atomic candidate containing the original InsertNode followed by EditText with the latest draft, without applying it or acknowledging insertion implicitly
