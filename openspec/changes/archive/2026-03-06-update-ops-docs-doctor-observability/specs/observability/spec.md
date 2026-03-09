## ADDED Requirements

### Requirement: Backend Exposes Liveness and Readiness Endpoints
The backend SHALL expose:
- a liveness endpoint that indicates the process is running, and
- a readiness endpoint that indicates the Hub can serve requests (including database connectivity).

The endpoints MUST return a small JSON document with an `ok` boolean.

#### Scenario: Readiness reflects DB availability
- **WHEN** the Hub database cannot be opened
- **THEN** the readiness endpoint returns `ok=false`
