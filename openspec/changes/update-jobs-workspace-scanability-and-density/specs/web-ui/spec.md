## ADDED Requirements

### Requirement: Jobs List Shows Latest Run Status And Time
In the Jobs workspace, the jobs list SHALL display each job's latest run status and latest run time (or an explicit empty state) to improve scanability.

#### Scenario: Desktop list shows status and time without increasing navigation
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the jobs list is displayed
- **THEN** each job row shows the latest run status (success/failed/running/queued/rejected as applicable)
- **AND** each job row shows the latest run time (or a clear "never ran" indication)

#### Scenario: Mobile list remains readable
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the jobs list is displayed
- **THEN** status and time are shown in a compact layout that does not force horizontal scrolling

### Requirement: Overview Shows Run Policy Strip
In the Jobs workspace Overview section, the UI SHALL show a compact run policy strip that includes schedule, schedule timezone, and overlap policy.

#### Scenario: Policy is visible without opening the editor
- **GIVEN** the user is viewing a job Overview
- **WHEN** the job has schedule configuration
- **THEN** the Overview shows schedule and timezone in the policy strip
- **AND** the Overview shows the overlap policy in the policy strip

#### Scenario: Policy strip wraps on mobile
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the Overview is rendered
- **THEN** the policy strip wraps naturally and remains fully usable

### Requirement: Overview Uses Compact Metadata Cards With Prominent Values
In the Jobs workspace Overview section, the UI SHALL present configuration metadata cards in a compact format while making the primary values visually prominent.

The cards SHALL include at least:
- source type,
- target type,
- backup format, and
- encryption.

#### Scenario: Values are emphasized while preserving vertical density
- **GIVEN** the user is viewing a job Overview
- **WHEN** metadata cards are rendered
- **THEN** labels are visually secondary
- **AND** values are visually prominent (larger typography and/or stronger emphasis)
- **AND** the cards do not waste vertical space

### Requirement: Format And Encryption Are Presented With Friendly Labels
In the Jobs workspace Overview section, backup format and encryption SHALL be presented using user-friendly labels, with optional code details where helpful.

#### Scenario: Format label and code are both available
- **GIVEN** the job uses a supported backup format
- **WHEN** the Overview renders the format card
- **THEN** the UI shows a friendly format label
- **AND** the UI MAY show the underlying format code as a secondary detail

#### Scenario: Encryption status is explicit
- **GIVEN** the job supports encryption
- **WHEN** the Overview renders the encryption card
- **THEN** the UI clearly indicates whether encryption is enabled or disabled
- **AND** when enabled, the UI shows which encryption key is selected as a secondary detail

### Requirement: History Provides Quick Status Filters
In the Jobs workspace History section, the UI SHALL provide quick status filter chips to allow narrowing run history without requiring a separate filter form.

#### Scenario: User filters to failures quickly
- **GIVEN** the user is viewing job History
- **WHEN** the user selects the "Failed" filter chip
- **THEN** the runs list shows only failed runs

#### Scenario: Filters remain usable on mobile
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the History filters are rendered
- **THEN** filter chips remain usable without horizontal overflow

### Requirement: Data Section Provides Guardrails For Destructive Actions
In the Jobs workspace Data section, the UI SHALL provide clear guardrails for destructive actions (such as retention apply and bulk delete) by showing warning text and scope hints near the actions.

#### Scenario: Retention action includes a warning and scope hint
- **GIVEN** the user is viewing the job Data section
- **WHEN** retention actions are available
- **THEN** the UI shows warning text describing impact (deleting snapshots) near the retention actions
- **AND** the UI indicates the scope is limited to the current job

#### Scenario: Bulk delete action includes a warning and scope hint
- **GIVEN** the user has selected snapshots to delete
- **WHEN** the delete action is available
- **THEN** the UI shows warning text describing impact (permanent deletion) near the delete action
- **AND** the UI indicates the scope is limited to the selected snapshots for the current job

### Requirement: Workbench Scroll Containers Provide Scrollability Cues
In the Jobs workspace on desktop-sized screens, scroll containers inside the workbench SHALL provide subtle cues (such as shadows/fades) indicating scrollability and scroll position.

#### Scenario: Jobs list pane shows scroll cues when overflowing
- **GIVEN** the jobs list pane content exceeds the pane height
- **WHEN** the user scrolls the jobs list pane
- **THEN** the UI shows subtle cues indicating additional content above/below

#### Scenario: Job content pane shows scroll cues when overflowing
- **GIVEN** the job content pane exceeds the pane height
- **WHEN** the user scrolls the job content pane
- **THEN** the UI shows subtle cues indicating additional content above/below

