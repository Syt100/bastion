## ADDED Requirements

### Requirement: System Status Includes Build Metadata
The backend SHALL expose build metadata via `/api/system` so the Web UI can display Hub version and build time.

#### Scenario: System status includes build time
- **WHEN** a client requests `/api/system`
- **THEN** the response includes `version` and `build_time_unix`

#### Scenario: Source build without git still works
- **GIVEN** the Hub is built from source without a `.git` directory
- **WHEN** a client requests `/api/system`
- **THEN** the endpoint still returns successfully
- **AND** build metadata may fall back to `unknown`
