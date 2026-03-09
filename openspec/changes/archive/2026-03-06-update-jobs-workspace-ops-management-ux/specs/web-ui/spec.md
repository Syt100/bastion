## ADDED Requirements

### Requirement: Jobs Workspace Provides Clear Refresh Controls
The Web UI SHALL provide distinct, accessible refresh actions for:
- refreshing the jobs list, and
- refreshing the selected job's detail.

#### Scenario: User can tell list refresh apart from detail refresh
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user is in the Jobs workspace in split layout
- **AND** a job is selected (detail is visible)
- **WHEN** the user views the available refresh actions
- **THEN** the UI provides clear labels and/or tooltips that distinguish refreshing the list from refreshing the selected job
- **AND** both actions provide accessible labels (e.g. `aria-label`)

### Requirement: Jobs Workspace Shows Active Filters and Result Counts
The Web UI SHALL show a compact summary of the jobs list filtering state, including:
- a results counter (filtered count and total count), and
- active filter chips that can be removed individually, and
- a clear-all filters action.

#### Scenario: User removes one active filter chip
- **GIVEN** the user has applied at least one jobs list filter
- **WHEN** the user closes an active filter chip
- **THEN** only that corresponding filter is cleared
- **AND** the jobs list results update accordingly

### Requirement: Jobs Workspace Supports Bulk Selection and Safe Bulk Actions
In list-only layout on desktop-sized screens, the Web UI SHALL allow selecting multiple jobs and performing safe bulk actions:
- **Run now**
- **Archive**
- **Unarchive**

The UI SHALL show a selection toolbar when one or more jobs are selected.

#### Scenario: Bulk run now skips archived jobs
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** the user selects multiple jobs including at least one archived job
- **WHEN** the user triggers bulk Run now
- **THEN** the UI skips archived jobs
- **AND** the UI reports a summary outcome (queued/rejected/skipped/failed)

#### Scenario: Bulk archive requires confirmation
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** the user selects one or more non-archived jobs
- **WHEN** the user triggers bulk Archive
- **THEN** the UI asks for confirmation before archiving
- **AND** the UI provides an option to cascade snapshot archival when supported

### Requirement: Jobs Table View Improves Sorting and Column Affordances
In jobs list-only Table view on desktop-sized screens, the Web UI SHALL:
- support header click sorting for at least job name and updated time,
- keep the header sort state and the sort control in sync, and
- keep key columns (at least name and actions) visible while horizontally scrolling.

#### Scenario: User sorts by clicking a table header
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** the jobs list is displayed in Table view
- **WHEN** the user clicks the Name column header to sort
- **THEN** the jobs list order updates
- **AND** the sort control reflects the same sort key and direction

### Requirement: Jobs Split View List Pane Is Resizable
On desktop-sized screens in split layout, the Web UI SHALL allow resizing the jobs list pane width via a drag handle.

The resized width SHALL be persisted on desktop-sized screens (local-only preference).

#### Scenario: User resizes the list pane and the width persists
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in split layout
- **WHEN** the user drags the split handle to resize the list pane
- **THEN** the list pane width updates immediately
- **AND** when the user reloads the page on desktop, the list pane uses the last selected width

### Requirement: Jobs List View Provides Quick Per-Row Actions
In Jobs list view on desktop-sized screens, the UI SHALL provide compact per-row actions without requiring opening the detail pane, including at least:
- Run now
- Edit
- More actions (overflow menu)

These actions SHOULD be visually de-emphasized until hover/focus to preserve scanability.

#### Scenario: Hover reveals per-row quick actions
- **GIVEN** the user is on a desktop-sized screen
- **AND** the jobs list is displayed in List view
- **WHEN** the user hovers a row (or focuses it via keyboard)
- **THEN** the UI reveals quick per-row actions for that row

### Requirement: Jobs List-only Layout Provides Clear Detail Access
When the Jobs workspace is in list-only layout and a job is selected, the UI SHALL provide a clear affordance to open the selected job in detail-only (or split) layout.

#### Scenario: User opens detail-only from list-only
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** a job is selected
- **WHEN** the user activates the "open details" affordance
- **THEN** the UI switches to a layout where job detail is visible
- **AND** the selected job remains selected

### Requirement: Mobile Job Detail Provides Sticky Actions
On mobile-sized screens, job detail actions SHALL be available without requiring the user to scroll back to the top of the page.

#### Scenario: User can run now while scrolled on mobile
- **GIVEN** the user is on a mobile-sized screen
- **AND** the user is viewing a job detail page
- **AND** the user has scrolled within the job detail content
- **WHEN** the user triggers Run now from the sticky actions area
- **THEN** the job run is triggered without requiring scrolling to the top

### Requirement: Jobs Workspace Filter Controls Have Accessible Names
The jobs list search input and filter controls SHALL provide stable accessible names (e.g. `name` attribute or `aria-label`) so they can be targeted reliably by automation and are not treated as unnamed form fields.

#### Scenario: Search and filters provide stable names
- **GIVEN** the user is on the Jobs workspace
- **WHEN** the UI renders the jobs list search and filter controls
- **THEN** each control provides a stable accessible name

