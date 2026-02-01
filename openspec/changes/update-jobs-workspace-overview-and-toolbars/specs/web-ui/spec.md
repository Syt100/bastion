## ADDED Requirements

### Requirement: Overview Shows Run Summary (Default Last 7 Days)
For a selected job, the Web UI SHALL show a run summary in the Overview section that defaults to the last 7 days.

#### Scenario: Overview shows latest run and 7-day counts
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/overview`
- **WHEN** run data exists for the job
- **THEN** the UI shows the latest run status and timestamp
- **AND** the UI shows compact run counts for the last 7 days (total, success, failed)
- **AND** the UI provides an action to open the latest run in the Run Detail drawer

#### Scenario: Overview handles jobs with no recent runs
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/overview`
- **WHEN** the job has no runs in the last 7 days
- **THEN** the UI shows a compact empty/zero state for the 7-day summary
- **AND** the UI does not offer a broken action to open a non-existent latest run

### Requirement: History Section Uses A Compact Header For Actions
The History section SHALL prioritize the runs list and SHALL place section actions (e.g. Refresh) in a compact header area rather than a standalone full-width action row.

#### Scenario: History actions do not consume a separate row
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/history`
- **WHEN** the History section is rendered
- **THEN** the runs list is shown as the primary content
- **AND** History actions are presented in the list panel header area
- **AND** the UI does not reserve a separate action row solely for a Refresh button

### Requirement: Data Section Uses Compact Per-Panel Actions
The Data section SHALL place actions for retention and snapshots inside their respective panel headers to reduce vertical space, especially on mobile.

#### Scenario: Retention save action is in the retention panel header
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/data`
- **WHEN** the retention panel is rendered
- **THEN** the primary Retention action (Save) is available in the retention panel header

#### Scenario: Snapshots refresh action is in the snapshots panel header
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/data`
- **WHEN** the snapshots list is rendered
- **THEN** the snapshots Refresh action is available in the snapshots panel header

### Requirement: Mobile Toolbars Avoid Multi-Line Action Rows
On mobile-sized screens, job section actions (History/Data) SHALL avoid layouts that introduce additional standalone action rows or multi-line toolbars.

#### Scenario: Mobile shows compact actions without wrapping
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user views History or Data
- **THEN** actions are presented as compact icon/overflow controls in headers
- **AND** the UI avoids adding a separate action row that pushes primary content below the fold
