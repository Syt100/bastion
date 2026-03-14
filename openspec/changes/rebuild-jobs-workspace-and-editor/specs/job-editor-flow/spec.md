## ADDED Requirements

### Requirement: Job Authoring Uses A Full-Page Stepper Flow
Creating or editing a job SHALL use a dedicated full-page stepper flow instead of a modal-only editor.

#### Scenario: Create uses the same step structure as edit
- **WHEN** an operator starts creating a new job or editing an existing job
- **THEN** the UI SHALL open the same full-page stepper flow
- **AND** the flow SHALL present explicit authoring steps rather than a single modal form

#### Scenario: Authoring routes are directly addressable
- **WHEN** an operator opens job create or edit from navigation, a workspace action, or a shared link
- **THEN** the editor SHALL render under dedicated top-level authoring routes
- **AND** refreshing the page SHALL preserve the authoring flow instead of requiring the originating modal context

### Requirement: Stepper Flow Uses Explicit Authoring Stages
The full-page job editor SHALL organize authoring into explicit stages for backup-job configuration.

#### Scenario: Standard steps are available
- **WHEN** the full-page job editor renders
- **THEN** it SHALL provide stages for `Basic`, `Source`, `Target`, `Schedule & Retention`, `Security`, `Notifications`, and `Review`
- **AND** the operator SHALL be able to move forward and backward without losing already completed inputs

### Requirement: The Editor Supports Progressive Validation
The editor SHALL validate inputs progressively at step boundaries and during final review.

#### Scenario: Step validation blocks invalid progression
- **WHEN** the operator attempts to proceed past a step with invalid required fields
- **THEN** the editor SHALL surface field or section validation feedback in that step
- **AND** the flow SHALL NOT advance until the blocking validation errors are resolved or intentionally bypassed according to design

#### Scenario: Review step replays unresolved risks
- **WHEN** the operator opens the final review step
- **THEN** the editor SHALL summarize unresolved warnings, risky settings, or missing optional-but-important signals before submission

#### Scenario: Server-backed validation can block stale or invalid submissions
- **WHEN** the operator attempts to save a create or edit flow
- **THEN** the editor SHALL surface authoritative backend validation results in the relevant step or review summary
- **AND** a late backend validation failure SHALL NOT leave the user without a clear step to return to

### Requirement: The Editor Preserves Draft Progress
The full-page editor SHALL preserve in-progress authoring so interrupted sessions can resume safely.

#### Scenario: Operator resumes an interrupted draft
- **GIVEN** the operator has an unfinished create or edit flow
- **WHEN** they leave and later return to the flow
- **THEN** the editor SHALL restore the in-progress draft state or offer to resume it

#### Scenario: Successful save clears the persisted draft
- **GIVEN** the operator submits a create or edit flow successfully
- **WHEN** the save completes
- **THEN** the corresponding persisted draft SHALL be removed
- **AND** reopening the editor later SHALL start from fresh server state unless a new draft has been created

#### Scenario: Edit draft detects stale base job revision
- **GIVEN** an edit draft was created from an older job revision
- **WHEN** the operator later resumes that draft after the underlying job was updated elsewhere
- **THEN** the editor SHALL detect the revision mismatch before overwrite
- **AND** it SHALL offer resume-with-warning, discard-draft, or reload-from-live behavior instead of silently overwriting

### Requirement: The Editor Shows Live Configuration And Risk Summary
The editor SHALL keep a configuration summary and risk summary visible throughout authoring.

#### Scenario: Operator sees the effect of step changes while editing
- **WHEN** the operator changes source, target, schedule, retention, or security settings
- **THEN** the editor SHALL update a live summary of the effective configuration
- **AND** any derived risks or warnings SHALL be visible before the final submit step

#### Scenario: Notifications remain visible in the authoring summary
- **WHEN** the operator changes notification delivery mode or destinations
- **THEN** the editor summary SHALL reflect the resulting notification behavior
- **AND** review output SHALL distinguish inherited notifications from custom overrides

### Requirement: Mobile Authoring Uses The Same Stepper Model Without Modal Constraints
Mobile job authoring SHALL use the same stepper semantics as desktop while remaining fully usable on small screens.

#### Scenario: Mobile create/edit is full-screen
- **WHEN** a mobile operator creates or edits a job
- **THEN** the editor SHALL render as a dedicated full-screen flow
- **AND** it SHALL NOT require a large modal that constrains scrolling, review, or section transitions
