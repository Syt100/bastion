## ADDED Requirements

### Requirement: Schedule Editor Supports Simple and Cron Modes
The Web UI SHALL provide a schedule editor for jobs that supports manual mode, a guided simple mode for common schedules, and an advanced cron expression mode.

#### Scenario: User selects “Daily at 02:00”
- **WHEN** a user configures “Daily at 02:00” in simple mode
- **THEN** the UI generates the corresponding cron and saves it

### Requirement: Schedule Timezone Is Configurable Per Job
The Web UI SHALL allow users to select a timezone for schedule interpretation, defaulting to the Hub timezone.

#### Scenario: Default timezone is hub timezone
- **GIVEN** the Hub timezone is `UTC`
- **WHEN** a user opens the job editor
- **THEN** the timezone selector defaults to `UTC`

### Requirement: UI Communicates DST Behavior
The Web UI SHALL clearly communicate that schedules are interpreted in the selected timezone and that DST gaps are skipped and DST folds run once.

#### Scenario: DST help text is visible
- **WHEN** a user views the schedule configuration section
- **THEN** the UI shows a brief explanation of DST behavior

