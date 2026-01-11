---
## MODIFIED Requirements

### Requirement: Desktop Filter Bars Are Compact
The web UI SHALL render list-page filter bars compactly on desktop screens, while keeping mobile-friendly controls.

#### Scenario: Cleanup list is scannable on desktop
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user opens the incomplete run cleanup page
- **THEN** the table shows a compact set of default columns suitable for scanning
- **AND** long error text does not dominate the layout

### Requirement: Enum Filters Support Multi-Select
The web UI SHALL allow selecting multiple values for low-cardinality enum filters.

#### Scenario: Details view adapts by device
- **WHEN** the user opens task details from the cleanup list
- **THEN** the details appear in a modal dialog on desktop
- **AND** appear in a bottom drawer on mobile screens

#### Scenario: Error is shown as type + summary in the list
- **GIVEN** a task has a recorded error
- **THEN** the list shows the error kind/type and a short summary
- **AND** the full error is accessible in the details view

