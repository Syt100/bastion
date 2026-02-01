## ADDED Requirements

### Requirement: Jobs Workspace With Master-Detail Navigation
The Web UI SHALL provide a node-scoped Jobs workspace at `/n/:nodeId/jobs` that enables browsing jobs and working on a selected job without excessive page-to-page navigation.

#### Scenario: Desktop shows master-detail layout
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the UI shows a jobs list pane and a job workspace pane
- **AND** selecting a job updates the workspace pane without leaving the Jobs workspace route family

#### Scenario: Mobile uses single-column navigation
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the UI shows the jobs list
- **AND** selecting a job navigates to the job workspace
- **AND** the job workspace provides a clear “back to jobs list” affordance

### Requirement: Job Workspace Has Three Top-Level Sections
For a selected job, the Web UI SHALL provide exactly three top-level sections:
- Overview
- History
- Data

#### Scenario: Job workspace redirects to Overview by default
- **GIVEN** the user opens `/n/:nodeId/jobs/:jobId`
- **THEN** the UI navigates to `/n/:nodeId/jobs/:jobId/overview`

#### Scenario: Switching sections preserves job context
- **GIVEN** the user is viewing a job workspace
- **WHEN** the user switches between Overview, History, and Data
- **THEN** the selected job remains the same
- **AND** the job header remains visible to reinforce context

### Requirement: Run Detail Is A Job-Scoped Drawer Overlay
The Web UI SHALL present Run Detail as a job-scoped overlay route rendered as a drawer, rather than as a top-level standalone page.

#### Scenario: Opening a run does not leave the current job section
- **GIVEN** the user is viewing a job section (Overview, History, or Data)
- **WHEN** the user opens a run from that section
- **THEN** the UI navigates to the corresponding run overlay route under that section
- **AND** a drawer overlay opens with the run details
- **AND** closing the drawer returns the user to the same job section

#### Scenario: Drawer presentation matches device size
- **WHEN** the user opens a run overlay route on desktop
- **THEN** the run details are shown in a side drawer
- **WHEN** the user opens a run overlay route on mobile
- **THEN** the run details are shown in a full-screen drawer

### Requirement: Data Section Combines Snapshots And Retention
The job Data section SHALL combine:
- snapshot browsing/management, and
- retention policy editing/preview/apply,
in a single place so users do not need to bounce between separate pages to manage job data lifecycle.

#### Scenario: Retention is managed from the Data section
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/data`
- **WHEN** the user edits retention settings and saves
- **THEN** retention changes apply to the current job
- **AND** the user can preview/apply retention without navigating away from the Data section

### Requirement: Advanced Inspection Is Not A Top-Level Section
Advanced inspection (e.g. JSON view / raw job payload) SHALL be available from the job workspace, but SHALL NOT be presented as a top-level section alongside Overview/History/Data.

#### Scenario: JSON inspection is accessed via overflow actions
- **GIVEN** the user is viewing a job workspace
- **WHEN** the user opens the “More” actions menu
- **THEN** the user can access JSON/inspect functionality from that menu

## MODIFIED Requirements

### Requirement: Node-Scoped Job Detail Uses Workspace Sections
The existing node-scoped job detail experience at `/n/:nodeId/jobs/:jobId` SHALL be implemented as the job workspace described above, using sections (Overview/History/Data) instead of the previous multi-tab structure.

#### Scenario: Job detail no longer relies on a large set of sibling pages
- **WHEN** the user navigates within a job detail experience
- **THEN** navigation is limited to the three job sections and run drawer overlays

### Requirement: Links To Run Detail Are Job-Scoped
Links that open Run Detail from anywhere in the Web UI SHALL navigate to a job-scoped run overlay route that includes both `jobId` and `runId`.

#### Scenario: Dashboard recent run opens job-scoped run drawer
- **GIVEN** the Dashboard shows a recent run with `job_id`, `node_id`, and `run_id`
- **WHEN** the user opens that run
- **THEN** the UI navigates to `/n/:nodeId/jobs/:jobId/history/runs/:runId`

## REMOVED Requirements

### Requirement: Top-Level Run Detail Page
The Web UI SHALL provide a top-level Run Detail page at `/n/:nodeId/runs/:runId`.

**Reason**: Run Detail must preserve job context and be presented as a job-scoped drawer overlay.
**Migration**: Not required (backward compatibility is explicitly out of scope).

