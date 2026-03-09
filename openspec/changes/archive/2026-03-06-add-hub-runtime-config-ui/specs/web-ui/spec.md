## ADDED Requirements

### Requirement: Hub Runtime Config Page (Restart Required)
The Web UI SHALL provide a Hub-only runtime config page that supports viewing and editing selected configuration.

#### Scenario: User sees effective vs saved config
- **WHEN** the user opens the runtime config page
- **THEN** the page MUST display the current effective value for each field
- **AND** the saved (pending) value if present
- **AND** a clear indicator that changes require a restart to take effect

### Requirement: Read-only Display For Unsafe Fields
The Web UI SHALL display these fields as read-only:
- Bind host/port
- Trusted proxies
- Insecure HTTP

#### Scenario: Unsafe fields cannot be edited
- **WHEN** the user views the runtime config page
- **THEN** the UI MUST NOT allow editing of unsafe fields

### Requirement: Editable Fields For Safe Policy Settings
The Web UI SHALL allow editing of safe policy settings (persisted to DB and applied on restart):
- Hub timezone
- Run retention days
- Incomplete cleanup days
- Logging filter/file/rotation/keep-files

#### Scenario: Save prompts restart
- **WHEN** the user saves updated runtime config
- **THEN** the UI MUST confirm the save
- **AND** indicate a restart is required for the changes to take effect

