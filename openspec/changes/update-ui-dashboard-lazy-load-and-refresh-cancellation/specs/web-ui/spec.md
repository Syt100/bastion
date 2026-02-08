## ADDED Requirements

### Requirement: Dashboard Defers Heavy Chart Module Loading
The dashboard view SHALL defer loading heavy visualization modules until chart data needs to be rendered, and SHALL show a loading fallback while the chart module resolves.

#### Scenario: Dashboard trend chart is lazy-loaded
- **GIVEN** a user navigates to the dashboard
- **AND** trend data exists for the 7-day chart
- **WHEN** the dashboard view renders
- **THEN** the chart visualization component is loaded asynchronously
- **AND** the dashboard shows a loading fallback until the chart module is ready

### Requirement: List Refresh Cancels Superseded Requests
For list refresh workflows in Web UI state stores, the system SHALL cancel superseded in-flight refresh requests when a newer refresh starts.

#### Scenario: New refresh cancels previous in-flight request
- **GIVEN** a list view triggers refresh request A
- **AND** request A is still in flight
- **WHEN** the user triggers refresh request B
- **THEN** request A is canceled
- **AND** only request B may update the active list state
