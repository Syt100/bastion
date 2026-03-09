## ADDED Requirements

### Requirement: Run-event detail dialogs SHALL prioritize concise diagnostics before verbose payloads
The Web UI SHALL present actionable diagnostics (localized message/hint, key envelope rows, and concise context) before rendering verbose raw payload sections.

#### Scenario: Event detail opens with envelope and raw fields
- **GIVEN** a run event contains `error_envelope` and additional raw `fields`
- **WHEN** a user opens event details from either run-detail page or run-events modal
- **THEN** the dialog SHALL show concise diagnostics first
- **AND** verbose raw JSON SHALL remain collapsed until the user explicitly expands it

### Requirement: Run-event detail dialogs SHALL support progressive disclosure for long error chains
The Web UI SHALL provide an expandable view for long `error_chain` diagnostics to reduce first-screen noise while keeping full detail available on demand.

#### Scenario: Long error chain exists
- **GIVEN** event fields include an `error_chain` list with multiple entries
- **WHEN** the detail dialog is rendered
- **THEN** the dialog SHALL show a compact preview first
- **AND** provide explicit expand/collapse actions to view full chain entries

### Requirement: Shared event-detail renderer SHALL keep behavior consistent across entry points
Run detail page and run-events modal SHALL reuse the same event-detail content behavior for diagnostics ordering, expansion controls, and raw payload rendering.

#### Scenario: User opens the same event from two entry points
- **GIVEN** an event is reachable from both run-detail and run-events modal
- **WHEN** user opens event details from each entry point
- **THEN** both dialogs SHALL render the same content sections and progressive disclosure controls
