## MODIFIED Requirements

### Requirement: Run Detail Shows Events and Linked Operations
The Run Detail page SHALL show live run events and a sub-list of linked operations (restore/verify) started from the run.

#### Scenario: User can filter and search events
- **GIVEN** a run has events
- **WHEN** the user applies filters (level/kind) or a search query
- **THEN** the list updates to show only matching events
- **AND** live events continue to arrive and are included when they match the filters

## ADDED Requirements

### Requirement: Run Detail Events Provide Quick Navigation and Export
The Run Detail page SHALL provide quick navigation and export helpers for run events.

#### Scenario: User jumps to first error
- **GIVEN** the events list contains an error event
- **WHEN** the user invokes "jump to first error"
- **THEN** the UI scrolls the list to the first error event

#### Scenario: User exports filtered events
- **GIVEN** the user has applied filters/search to the events list
- **WHEN** the user exports events
- **THEN** the UI exports the filtered events as JSON
