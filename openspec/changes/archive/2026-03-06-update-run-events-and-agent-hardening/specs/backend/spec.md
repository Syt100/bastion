## ADDED Requirements

### Requirement: Agent Run Ingest Enforces Size Limits
The backend SHALL enforce request body size limits for HTTP endpoints, including the Agent run ingest endpoint.

#### Scenario: Oversized ingest request is rejected
- **WHEN** an Agent sends a run ingest request exceeding the configured maximum size
- **THEN** the backend rejects the request with an appropriate HTTP error status (e.g., 413)

### Requirement: Agent Run Ingest Validates Payload and Is Idempotent
The backend SHALL validate Agent run ingest payloads and SHALL ingest in an idempotent manner.

#### Scenario: Ingest validates timestamps and required fields
- **WHEN** an Agent ingests a run record with invalid timestamps or missing required fields
- **THEN** the backend responds with a 4xx error and a stable error code

#### Scenario: Re-ingesting the same run does not create duplicates
- **WHEN** an Agent ingests the same run ID multiple times
- **THEN** the backend does not create duplicate run events for the same `(run_id, seq)`

### Requirement: Ingest Can Upsert Run Metadata
The backend SHALL support upserting run metadata (status/ended_at/summary/error) for an existing run ID during Agent ingest.

#### Scenario: Ingest updates an existing run record
- **WHEN** an Agent ingests a run ID that already exists in the database
- **THEN** the backend updates the run’s metadata to reflect the ingested payload

