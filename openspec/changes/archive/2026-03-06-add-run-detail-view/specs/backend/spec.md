## ADDED Requirements

### Requirement: Run Read API
The backend SHALL provide an authenticated API to fetch run details by id.

#### Scenario: Fetch an existing run
- **WHEN** the user requests `GET /api/runs/{run_id}`
- **THEN** the response includes run status, timestamps, job id, and any available summary/error fields

#### Scenario: Run not found
- **WHEN** the user requests `GET /api/runs/{run_id}` for an unknown id
- **THEN** the backend returns `404 run_not_found`

