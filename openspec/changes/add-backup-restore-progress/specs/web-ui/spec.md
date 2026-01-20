## ADDED Requirements

### Requirement: Filesystem Job Editor Supports Pre-Scan Toggle
The Web UI SHALL expose a filesystem job option to enable/disable pre-scan (`source.pre_scan`) and default it to enabled for new jobs.

#### Scenario: New filesystem job defaults pre-scan on
- **WHEN** the user opens the create-job dialog for a filesystem job
- **THEN** the pre-scan option is enabled by default

#### Scenario: User disables pre-scan
- **WHEN** the user disables pre-scan and saves the job
- **THEN** the saved job spec includes `source.pre_scan = false`

