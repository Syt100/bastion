## ADDED Requirements

### Requirement: Job Detail actions toolbar
The Web UI SHALL provide a job-level actions toolbar on the Job Detail page (`/n/:nodeId/jobs/:jobId`) so that common actions are accessible without switching to a secondary tab.

#### Scenario: User sees common actions on Job Detail
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId`
- **THEN** the UI shows a toolbar with job-level actions (run now, edit, deploy, archive/unarchive, delete)

#### Scenario: Destructive actions require confirmation
- **WHEN** the user attempts to archive or delete a job from the toolbar
- **THEN** the UI requires explicit confirmation before performing the action

