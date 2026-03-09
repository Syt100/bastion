---
## ADDED Requirements

### Requirement: UI Provides Incomplete Cleanup Management (Mobile Friendly)
The web UI SHALL provide a mobile-friendly page to view incomplete cleanup tasks and take operator actions.

#### Scenario: Mobile layout is usable
- **WHEN** the user opens the cleanup page on a small screen
- **THEN** tasks are displayed in a card layout with readable status, error, and actions without horizontal scrolling

### Requirement: UI Supports Archive vs Permanent Delete
The web UI SHALL let users choose between archiving a job and permanently deleting it.

#### Scenario: Archive is available from the delete flow
- **WHEN** the user attempts to delete a job
- **THEN** the UI offers an “Archive (keep history)” option and a “Delete permanently (cascade)” option

