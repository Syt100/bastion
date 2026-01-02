## ADDED Requirements

### Requirement: Per-Job Notification Override (Inherit or Custom)
Each job SHALL support notification configuration with two modes:
- `inherit`: use global/channel settings and all enabled destinations
- `custom`: per channel, choose destinations explicitly (multi-select)

Disabled destinations MAY be selected in `custom` mode, but SHALL be treated as not deliverable until enabled.

#### Scenario: Job inherits global notifications
- **WHEN** a job is created without an explicit notification override
- **THEN** the job inherits global notification settings

#### Scenario: Job uses a custom destination subset
- **WHEN** a job is configured to use WeCom destinations `A,B` and no Email destinations
- **THEN** only `A,B` receive WeCom notifications on run completion
- **AND** no Email notifications are sent for that job

