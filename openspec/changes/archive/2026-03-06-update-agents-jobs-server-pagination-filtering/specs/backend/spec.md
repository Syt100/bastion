## ADDED Requirements

### Requirement: Agents List API Supports Server-Side Pagination and Search/Status Filtering
The agents list API MUST support server-side pagination and filtering for status/search while preserving existing label filtering semantics.

#### Scenario: Agents list returns paged payload with metadata
- **GIVEN** an authenticated request to `/api/agents`
- **WHEN** the caller provides `page` and `page_size`
- **THEN** the response includes `items`, `page`, `page_size`, and `total`
- **AND** `items` contains only rows for the requested page

#### Scenario: Agents list filters by status and search query
- **GIVEN** agents with mixed online/offline/revoked states and distinct names/ids
- **WHEN** the caller provides `status` and `q`
- **THEN** only matching agents are returned
- **AND** `total` reflects the filtered count before pagination

### Requirement: Jobs List API Supports Server-Side Node-Scoped Filtering, Sorting, and Pagination
The jobs list API MUST support server-side filters for node scope, include-archived mode, latest-run status, schedule mode, and free-text search, plus deterministic sorting and page/page_size pagination.

#### Scenario: Jobs list returns one page with applied filters
- **GIVEN** an authenticated request to `/api/jobs`
- **WHEN** the caller specifies `node_id`, `include_archived`, `latest_status`, `schedule_mode`, `q`, `page`, and `page_size`
- **THEN** only matching jobs are returned in `items`
- **AND** `total` reports filtered result size before pagination

#### Scenario: Jobs list honors remote sort key
- **GIVEN** jobs with different names and update timestamps
- **WHEN** the caller provides a supported `sort` value
- **THEN** rows are ordered according to that sort mode
- **AND** ordering is stable for equal sort keys
