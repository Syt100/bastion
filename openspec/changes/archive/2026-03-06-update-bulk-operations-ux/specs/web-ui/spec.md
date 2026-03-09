## ADDED Requirements

### Requirement: Bulk Operation Detail Auto-Refresh
The Web UI SHALL automatically refresh the Bulk Operation detail view while an operation is running.

#### Scenario: Running operation refreshes without manual action
- **GIVEN** a bulk operation is in `running` status
- **WHEN** the user opens the operation detail view
- **THEN** the UI refreshes the operation detail automatically until it is no longer running

### Requirement: Failure-Focused Filtering
The Web UI SHALL allow focusing on failed items in a bulk operation.

#### Scenario: User filters to failed items
- **GIVEN** a bulk operation has failed items
- **WHEN** the user enables a “failed only” filter
- **THEN** the item list shows only failed items

