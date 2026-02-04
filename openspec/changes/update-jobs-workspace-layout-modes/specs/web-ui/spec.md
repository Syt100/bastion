## ADDED Requirements

### Requirement: Jobs Workspace Supports Desktop Layout Modes
On desktop-sized screens, the Web UI SHALL allow the user to switch the Jobs workspace layout between:
- **split**: jobs list pane + job workspace pane
- **list-only**: jobs list pane takes full width (job workspace hidden)
- **detail-only**: job workspace pane takes full width (jobs list hidden)

#### Scenario: User switches to list-only for ops management
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user is in the Jobs workspace
- **WHEN** the user switches to list-only layout
- **THEN** the jobs list expands to full width
- **AND** the job workspace pane is hidden
- **AND** the user can still select a job from the list (selection is reflected in the URL and selection highlight)

#### Scenario: User switches to detail-only for focused inspection
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user has a job selected in the Jobs workspace
- **WHEN** the user switches to detail-only layout
- **THEN** the job workspace expands to full width
- **AND** the jobs list pane is hidden
- **AND** the UI provides a clear affordance to return to split layout

#### Scenario: Detail-only does not show an empty pane
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user is at `/n/:nodeId/jobs` with no job selected
- **WHEN** the user attempts to switch to detail-only layout
- **THEN** the UI falls back to split layout or list-only layout
- **AND** the user is not shown an empty job workspace page

#### Scenario: Desktop layout preference is persisted
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user selects a workspace layout mode
- **WHEN** the user navigates within the Jobs route family or reloads the page
- **THEN** the Jobs workspace uses the user's last selected layout mode by default

#### Scenario: Mobile ignores desktop layout preferences
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the UI uses single-column navigation (jobs list, then job workspace with a back affordance)
- **AND** desktop layout mode controls are not shown

### Requirement: Jobs List Supports List/Table Views For Ops Management
In the Jobs workspace on desktop-sized screens, the jobs list SHALL support:
- a **List** view optimized for scanability, and
- a **Table** view optimized for management (sorting + columns).

The Table view SHALL be available only when the Jobs workspace is in list-only layout.

#### Scenario: Table view forces list-only layout
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user selects the Table view
- **THEN** the Jobs workspace switches to list-only layout
- **AND** the jobs list is rendered as a table

#### Scenario: Table view provides ops-friendly columns and sorting
- **GIVEN** the user is on a desktop-sized screen
- **AND** the jobs list is shown in Table view
- **THEN** the table provides columns for at least:
  - job name,
  - node (hub/agent),
  - schedule (and schedule timezone when relevant),
  - latest run status,
  - latest run time (or an explicit "never ran" indication),
  - last updated time, and
  - per-row actions
- **AND** the user can sort by at least job name and last updated time

#### Scenario: List view remains consistent across Split and List-only layouts
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the jobs list is rendered in List view
- **THEN** the row structure remains recognizably the same in Split and List-only layouts
- **AND** List-only MAY use more comfortable spacing without changing the core row shape

#### Scenario: Mobile does not expose Table view
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the jobs list is displayed
- **THEN** the Table view toggle is not shown

## MODIFIED Requirements

### Requirement: Jobs Workspace With Master-Detail Navigation
The Web UI SHALL provide a node-scoped Jobs workspace at `/n/:nodeId/jobs` that enables browsing jobs and working on a selected job without excessive page-to-page navigation.

#### Scenario: Desktop uses the user's preferred layout mode (default split)
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the UI shows the jobs list
- **AND** the UI shows or hides the job workspace pane based on the user's preferred layout mode (default: split)
- **AND** selecting a job updates the job selection in the route without leaving the Jobs workspace route family

