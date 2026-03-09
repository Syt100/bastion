## ADDED Requirements

### Requirement: Dashboard Overview Page (Metrics + Trend + Recent Runs)
The Web UI SHALL provide a Dashboard page that surfaces a high-level overview of backup status and recent activity.

#### Scenario: Dashboard shows overview (no checklist)
- **WHEN** the user opens the Dashboard page
- **THEN** the page shows overview sections (stats, trend, recent runs)
- **AND** the page does not show a setup checklist

#### Scenario: Dashboard works when there is no data yet
- **GIVEN** there are no runs yet
- **WHEN** the user opens the Dashboard page
- **THEN** the Dashboard shows zero/empty values without errors
- **AND** the recent runs section displays an empty-state message

#### Scenario: Recent runs list links to Run Detail
- **GIVEN** there is a run in the recent runs list
- **WHEN** the user clicks it
- **THEN** the UI navigates to the Run Detail page for that run

