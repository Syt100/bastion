## MODIFIED Requirements

### Requirement: Run Detail Shows Consistency Warnings
When a run includes source consistency warnings, the Web UI SHALL present an explicit warning in the run detail view.

#### Scenario: Run detail shows warning tag
- **WHEN** a user opens a run detail page whose summary contains consistency warnings
- **THEN** the UI displays a warning badge/tag indicating the backup may be inconsistent

### Requirement: Job Runs List Shows Consistency Warnings
When a run includes source consistency warnings, the Web UI SHALL surface a warning indicator directly in the job runs list.

#### Scenario: Job runs list shows warning tag
- **WHEN** a user views a job's runs list
- **THEN** runs with `consistency_changed_total > 0` display a warning badge/tag
- **AND** the tag includes a count or clear indicator that changes were detected

