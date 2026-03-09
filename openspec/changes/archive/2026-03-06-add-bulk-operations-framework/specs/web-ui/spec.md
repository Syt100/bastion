## ADDED Requirements

### Requirement: UI Provides Bulk Operations Panel
The Web UI SHALL provide a panel/page to view bulk operations and their per-node results.

#### Scenario: Operator can see progress and failures
- **WHEN** the user opens a bulk operation
- **THEN** the UI MUST show overall progress and per-node results
- **AND** MUST display error summaries for failed items

### Requirement: UI Supports Retry Failed and Cancel
The Web UI SHALL allow an operator to retry failed items and cancel an operation.

#### Scenario: Retry failed is available from UI
- **GIVEN** an operation has failed items
- **WHEN** the user clicks “retry failed”
- **THEN** the UI MUST trigger the backend retry API and refresh status

#### Scenario: Cancel is available from UI
- **WHEN** the user clicks “cancel”
- **THEN** the UI MUST trigger the backend cancel API and refresh status

### Requirement: Agents Page Can Start Bulk Label Updates
The Web UI SHALL provide an entry point on the Agents page to start bulk label operations (add/remove labels) using a node selector.

#### Scenario: User starts bulk label add
- **GIVEN** the user has selected a set of nodes (explicit selection or label filter)
- **WHEN** the user starts a bulk “add labels” operation
- **THEN** the UI MUST create a bulk operation and show its progress/results

