## MODIFIED Requirements

### Requirement: Node-Scoped Run Detail Page
The Web UI SHALL provide a node-scoped Run Detail page at `/n/:nodeId/runs/:runId`.

#### Scenario: User opens Run Detail from the runs list
- **GIVEN** a user is viewing a job’s run list
- **WHEN** the user selects a run
- **THEN** the UI navigates to the Run Detail page for that run

#### Scenario: Run Detail presents a clear “status + key facts + actions” header
- **GIVEN** a run detail is loaded
- **THEN** the page shows the run status as a prominent badge near the title
- **AND** the run id is displayed as secondary information with a copy affordance
- **AND** primary actions are visually separated from secondary actions

### Requirement: Run Detail Shows Events and Linked Operations
The Run Detail page SHALL show live run events and a sub-list of linked operations (restore/verify) started from the run.

#### Scenario: Restore operation remains visible after closing dialogs
- **WHEN** a user starts a restore from Run Detail
- **THEN** the resulting restore operation appears in the operations sub-list for that run

#### Scenario: Operations empty state is compact and readable
- **GIVEN** a run has no linked operations
- **THEN** the operations section shows a compact empty state
- **AND** it does not render a large empty table that dominates the page

## ADDED Requirements

### Requirement: Run Detail Uses Responsive “Overview + Progress” Layout
The Run Detail page SHALL present the overview and progress information in a responsive layout that is readable on both desktop and mobile.

#### Scenario: Desktop uses a two-column first screen
- **GIVEN** the user is on a desktop viewport
- **THEN** the Run Detail page shows “Overview” and “Progress” side-by-side

#### Scenario: Mobile uses a single-column first screen
- **GIVEN** the user is on a mobile viewport
- **THEN** the Run Detail page stacks “Overview” above “Progress”

### Requirement: Run Detail Events Are Scan-Friendly
The Run Detail page SHALL present run events in a scan-friendly list.

#### Scenario: Events are shown as a timeline list with details
- **GIVEN** a run has events
- **THEN** the page shows events in a list optimized for scanning (timestamp + level + message)
- **AND** users can open event details to view any structured fields

### Requirement: Run Summary Shows Highlights and Raw JSON
The Run Detail page SHALL show a readable summary with an option to view the raw JSON.

#### Scenario: Summary shows structured highlights with a raw JSON fallback
- **GIVEN** a run has a summary payload
- **THEN** the page shows key summary highlights in a readable format
- **AND** the raw JSON is available via a collapsible section with a copy affordance

