## ADDED Requirements

### Requirement: Agents View Uses Remote Pagination and Remote Core Filters
Agents view SHALL request paginated agent data from backend using current page/page_size and current filter state (labels, labels mode, status, search query) instead of client-side full-list filtering.

#### Scenario: Changing page triggers remote fetch
- **GIVEN** the agents list is displayed
- **WHEN** the user changes page or page size
- **THEN** the view requests `/api/agents` with updated pagination query params
- **AND** pagination UI uses server `total` to render controls

#### Scenario: Changing status or search triggers remote fetch
- **GIVEN** agents list filters are visible
- **WHEN** the user updates status/search filters
- **THEN** the view resets to page 1 and requests server-filtered data
- **AND** list rendering uses returned `items` directly

### Requirement: Jobs Workspace List Uses Remote Filtering/Sorting/Pagination
Jobs workspace list SHALL execute filtering, sorting, and pagination through `/api/jobs` query params based on current node context and filter controls.

#### Scenario: List filters update backend query
- **GIVEN** the jobs workspace list is open
- **WHEN** the user changes archived toggle/search/latest-status/schedule/sort or node context
- **THEN** the view refreshes from backend with matching query params
- **AND** list/table render only returned page items

#### Scenario: Pagination controls reflect backend total
- **GIVEN** the backend reports paged list metadata
- **WHEN** the jobs workspace renders pagination
- **THEN** `item-count` is sourced from backend `total`
- **AND** page changes request the corresponding server page
