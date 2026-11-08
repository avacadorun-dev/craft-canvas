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
