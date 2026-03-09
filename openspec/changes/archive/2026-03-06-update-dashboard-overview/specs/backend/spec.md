## ADDED Requirements

### Requirement: Dashboard Overview API
The backend SHALL provide an authenticated API endpoint to return a dashboard overview payload for the Web UI.

#### Scenario: Auth is required
- **WHEN** a client requests `GET /api/dashboard/overview` without a valid session
- **THEN** the server responds with `401 Unauthorized`

#### Scenario: The response includes a 7-day trend series
- **WHEN** a client requests `GET /api/dashboard/overview` with a valid session
- **THEN** the response includes a 7-day trend series for success/failed runs

#### Scenario: The endpoint is safe
- **WHEN** the endpoint returns recent runs and summary stats
- **THEN** it MUST NOT include secret values (credentials, tokens, encryption keys)

