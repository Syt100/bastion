## ADDED Requirements

### Requirement: Web UI for Jobs and Runs
The system SHALL provide a Web UI to create/edit jobs, trigger runs, and view run history and details.

#### Scenario: User triggers a run from the UI
- **WHEN** a user clicks "Run now" on a job
- **THEN** a new run is created and its status is visible in the UI

### Requirement: Web UI Internationalization (i18n)
The Web UI SHALL default to Simplified Chinese (`zh-CN`) and SHALL support switching between Simplified Chinese (`zh-CN`) and English (`en-US`), persisting the selection.

#### Scenario: Default language is Simplified Chinese
- **WHEN** a user opens the Web UI for the first time
- **THEN** the UI is displayed in `zh-CN`

#### Scenario: User switches language
- **WHEN** a user selects `en-US` from the language selector
- **THEN** the UI updates to English and persists the selection for future visits

### Requirement: Live Run Events
The Web UI SHALL display live run events/logs during execution.

#### Scenario: User watches live logs
- **WHEN** a run is executing
- **THEN** the UI receives and displays live events/log lines

### Requirement: Restore Wizard
The Web UI SHALL provide a restore wizard to select a restore point and restore destination and choose a conflict strategy.

#### Scenario: Restore to a new directory
- **WHEN** the user selects a restore point and destination directory
- **THEN** the system restores backup contents according to selected conflict strategy

### Requirement: Restore Drill Verification Wizard
The Web UI SHALL provide a verification wizard to run restore drills and view results.

#### Scenario: Run a restore drill
- **WHEN** the user starts a restore drill for a completed run
- **THEN** the system performs a full restore drill and reports pass/fail with details
