---
## ADDED Requirements

### Requirement: Desktop Filter Bars Are Compact
The web UI SHALL render list-page filter bars compactly on desktop screens, while keeping mobile-friendly controls.

#### Scenario: Cleanup page filters are compact on desktop
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user opens the incomplete run cleanup page
- **THEN** the filter controls are displayed in a compact horizontal layout

#### Scenario: Notifications queue filters are compact on desktop
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user opens the notifications queue page
- **THEN** the filter controls are displayed in a compact horizontal layout

### Requirement: Enum Filters Support Multi-Select
The web UI SHALL allow selecting multiple values for low-cardinality enum filters.

#### Scenario: Cleanup page supports multi-select status and target
- **WHEN** the user selects multiple statuses and targets on the cleanup page
- **THEN** the list shows tasks that match any selected status AND any selected target

#### Scenario: Notifications queue supports multi-select status and channel
- **WHEN** the user selects multiple statuses and channels on the notifications queue page
- **THEN** the list shows items that match any selected status AND any selected channel
