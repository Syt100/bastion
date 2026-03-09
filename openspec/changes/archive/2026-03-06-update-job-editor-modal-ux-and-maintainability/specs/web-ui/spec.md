## MODIFIED Requirements

### Requirement: Job Editor Modal Is Modular and Maintainable
The web UI SHALL implement the job create/edit modal using a small orchestration component and modular step components. API ↔ form mapping and validation SHALL be implemented as reusable utilities with unit tests.

#### Scenario: Mapping behavior is covered by unit tests
- **GIVEN** the job editor supports legacy job specs and multiple job types/targets
- **WHEN** the mapping utilities are changed
- **THEN** unit tests detect regressions for legacy fields and required mappings

### Requirement: Job Editor Provides Guided, Step-Based Validation
The web UI SHALL validate required fields per step. When validation fails, the UI SHALL show inline field feedback AND automatically scroll/focus the first invalid field in the modal.

#### Scenario: Prevent progressing when a required field is missing
- **GIVEN** the user is creating a filesystem job
- **WHEN** the user clicks “Next” with an empty source paths list
- **THEN** the source paths field shows an error
- **AND** the modal scrolls to the source paths field and focuses it

#### Scenario: Cron is validated when provided
- **GIVEN** the user enters a cron schedule value
- **WHEN** the value is not a valid 5-field cron expression
- **THEN** the schedule field shows an error
- **AND** the modal scrolls/focuses the schedule field

### Requirement: Job Editor Navigation and Actions Work Well on Mobile
The web UI SHALL keep the job editor action bar persistently accessible (modal footer) on mobile. Step navigation SHALL allow returning to completed steps and SHALL prevent skipping ahead when prior steps are invalid.

#### Scenario: Action bar remains accessible on small screens
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user scrolls within a long editor step
- **THEN** the action buttons (Back/Next/Save) remain accessible without scrolling to the bottom

### Requirement: Job Editor Offers Shortcuts to Manage Related Settings
The web UI SHALL provide quick links from the job editor to manage WebDAV secrets and notification destinations. These links SHOULD open in a new tab to preserve the current draft.

#### Scenario: Open settings to manage WebDAV secrets
- **GIVEN** the user is configuring a WebDAV target in the job editor
- **WHEN** the user clicks the “Manage WebDAV secrets” link
- **THEN** the WebDAV secrets settings page opens

## ADDED Requirements

### Requirement: Job Editor Provides Common Cron Presets
The web UI SHALL provide a small set of common cron presets to quickly populate the schedule field.

#### Scenario: Apply a common schedule preset
- **GIVEN** the user is editing the job schedule
- **WHEN** the user selects a “daily at midnight” preset
- **THEN** the schedule field is populated with the preset value
