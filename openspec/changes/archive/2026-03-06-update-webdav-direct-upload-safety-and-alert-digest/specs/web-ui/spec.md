## MODIFIED Requirements

### Requirement: UI Exposes WebDAV Raw-Tree Direct Upload Controls
When editing a filesystem job that targets WebDAV and uses raw-tree format, the Web UI SHALL expose explicit direct upload configuration:
- `off|auto|on`
- advanced safety limits (concurrency + rate limits)

#### Scenario: User configures direct upload mode
- **WHEN** a user edits a filesystem job (WebDAV + raw-tree)
- **THEN** the direct upload mode can be set and saved

### Requirement: Runs List Shows High-Signal Warning Summary Only
The job runs list SHALL display a capped set of high-signal warning badges derived from the runs list digest fields, while run detail continues to show full samples and evidence.

#### Scenario: Low-signal warnings are de-noised
- **WHEN** a run has only low-volume changed-only consistency warnings
- **THEN** the runs list does not display a consistency warning badge

#### Scenario: High-signal warnings are visible
- **WHEN** a run has errors, or high-signal consistency warnings (replaced/deleted/read_error)
- **THEN** the runs list displays badges for those high-signal warnings

