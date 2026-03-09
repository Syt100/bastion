## ADDED Requirements

### Requirement: Desktop App Shell Uses Fixed Navigation With Content-Only Scrolling
On desktop-sized screens, the Web UI SHALL keep the App Shell header and main navigation visible while the user scrolls long content.

#### Scenario: Long page does not scroll the header and sider away
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user navigates to a page whose content exceeds the viewport height
- **AND** the user scrolls
- **THEN** the header remains visible
- **AND** the main navigation sider remains visible
- **AND** only the main content region scrolls

### Requirement: Jobs Workspace Desktop Uses Pane-Scoped Scrolling
In the Jobs workspace on desktop-sized screens, the Web UI SHALL implement pane-scoped scrolling so that the jobs list pane and the job workspace pane can be scrolled independently.

#### Scenario: Scrolling job content does not hide the jobs list pane
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user is viewing the Jobs workspace with a selected job
- **WHEN** the user scrolls through a long job section (History/Data)
- **THEN** the jobs list pane remains available without requiring the user to scroll back to the top of the page

### Requirement: Jobs List Filters Are Pinned Within The Jobs List Pane
In the Jobs workspace on desktop-sized screens, the jobs list pane SHALL keep its filter/search/sort controls visible while the user scrolls the jobs list.

#### Scenario: Filters remain accessible while scrolling the job list
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the jobs list contains enough entries to require scrolling
- **THEN** the jobs list can be scrolled
- **AND** the filter/search/sort controls remain visible within the list pane

### Requirement: Job Context Header And Section Navigation Are Pinned Within The Job Workspace Pane
In the Jobs workspace on desktop-sized screens, the job workspace pane SHALL keep the job context header and section navigation (Overview/History/Data) visible while the user scrolls section content.

#### Scenario: Job context remains visible while scrolling a long section
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user is viewing a job section whose content exceeds the viewport height
- **WHEN** the user scrolls the job workspace pane
- **THEN** the job header remains visible
- **AND** the section navigation remains visible

## MODIFIED Requirements

### Requirement: Jobs Workspace With Master-Detail Navigation
The Web UI SHALL provide a node-scoped Jobs workspace at `/n/:nodeId/jobs` that enables browsing jobs and working on a selected job without excessive page-to-page navigation.

#### Scenario: Desktop shows master-detail layout
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the UI shows a jobs list pane and a job workspace pane
- **AND** selecting a job updates the workspace pane without leaving the Jobs workspace route family
- **AND** long job content does not cause the jobs list pane to scroll out of view

#### Scenario: Mobile uses single-column navigation
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the UI shows the jobs list
- **AND** selecting a job navigates to the job workspace
- **AND** the job workspace provides a clear “back to jobs list” affordance
