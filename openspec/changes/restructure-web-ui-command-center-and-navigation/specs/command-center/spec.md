## ADDED Requirements

### Requirement: Command Center Prioritizes Operational Attention
The Web UI SHALL provide a `Command Center` landing page that prioritizes actionable operational attention over generic overview counters.

#### Scenario: Landing page opens on Command Center
- **WHEN** an authenticated operator opens the primary landing page
- **THEN** the first visible sections SHALL emphasize risks, critical activity, and recovery readiness
- **AND** generic counters SHALL be visually subordinate to those attention-oriented sections

#### Scenario: Healthy systems render a quiet but non-empty state
- **GIVEN** there are no failing runs, offline agents, or failed notification items
- **WHEN** the operator opens the Command Center
- **THEN** the page SHALL render an explicit healthy-state summary
- **AND** it SHALL NOT replace the page with an empty placeholder or zero-only KPI grid

### Requirement: Command Center Uses A Dedicated Aggregated Read Model
The system SHALL provide a dedicated Command Center view model that returns the landing page's major sections as one scope-aware response.

#### Scenario: Aggregated response echoes effective scope and range
- **WHEN** an authenticated client requests Command Center data
- **THEN** the response SHALL include both the requested and effective scope context
- **AND** it SHALL include the active range or time-window context used to generate the aggregates

#### Scenario: Section-level degradation does not invalidate the whole response
- **GIVEN** one Command Center section cannot be fully populated
- **WHEN** the aggregated response is returned
- **THEN** the overall response SHALL remain valid when the remaining sections can still be rendered
- **AND** the degraded section SHALL carry explicit state so the UI can show a partial-data message instead of failing the whole page

### Requirement: Attention Items Are Prioritized And Actionable
The system SHALL group and rank attention items by severity and recency, and each item SHALL expose at least one direct follow-up action.

#### Scenario: Failed run appears as a direct-action item
- **GIVEN** a recent run has terminal `failed` status
- **WHEN** the Command Center data is rendered
- **THEN** the run SHALL appear in the attention list with failure summary metadata
- **AND** the item SHALL provide a direct action that opens the corresponding run or owning job

#### Scenario: Offline agent appears with fleet navigation
- **GIVEN** an agent is offline or revoked
- **WHEN** the Command Center data is rendered
- **THEN** the attention list SHALL include the agent issue with severity metadata
- **AND** the item SHALL provide a direct action to open the Fleet surface or the affected agent detail

#### Scenario: Attention actions always use canonical stable routes
- **WHEN** the Command Center renders a follow-up action for a run, job, fleet member, integration issue, or system task
- **THEN** the action target SHALL use the canonical top-level route family for that object or surface
- **AND** the Command Center SHALL NOT emit new links that depend on legacy node-scoped route prefixes

### Requirement: Command Center Shows Recent Critical Activity
The Command Center SHALL expose a recent-activity section focused on critical or operator-relevant events such as failures, restores, verifies, and other noteworthy operations.

#### Scenario: Critical activity excludes low-signal noise
- **GIVEN** the system has many recent routine successes plus a smaller number of failed or operator-initiated runs/operations
- **WHEN** the recent activity section is rendered
- **THEN** the section SHALL prioritize failed or otherwise noteworthy events
- **AND** routine low-signal activity SHALL NOT crowd out the critical items

### Requirement: Command Center Summarizes Recovery Readiness
The Command Center SHALL provide a recovery-readiness summary that helps operators judge whether backups appear restorable and recent enough.

#### Scenario: Readiness shows backup and verify health
- **GIVEN** recent backup and verify data exists
- **WHEN** the operator views the Command Center
- **THEN** the readiness section SHALL summarize at least recent successful backup state and recent verification state
- **AND** degraded readiness SHALL be visible without navigating to another page

#### Scenario: Readiness remains meaningful when verification has not been used
- **GIVEN** the system has successful backups but no recent verify activity
- **WHEN** the operator views the readiness section
- **THEN** the section SHALL indicate the missing verification signal explicitly
- **AND** it SHALL NOT falsely imply full recovery confidence

### Requirement: Command Center Supports Scope-Aware Aggregation
The Command Center SHALL render data relative to explicit UI scope selection and any supported time-range controls.

#### Scenario: Scope change updates all sections consistently
- **GIVEN** the operator changes the active scope from global or preferred scope to a specific node
- **WHEN** the Command Center refreshes
- **THEN** attention items, recent activity, readiness, and watchlist summaries SHALL all reflect the same effective scope

#### Scenario: Empty scope data still returns a valid view model
- **GIVEN** the selected scope has no runs, agents, or notifications yet
- **WHEN** the Command Center data is requested
- **THEN** the system SHALL return a valid empty-state view model
- **AND** the UI SHALL render the scoped empty state without errors

#### Scenario: All-nodes scope is supported explicitly
- **GIVEN** the operator selects `all` scope in the shell
- **WHEN** the Command Center data is requested
- **THEN** the aggregation SHALL span hub and agent-backed activity according to the active time range
- **AND** the response SHALL identify that `all` scope explicitly rather than implying a single-node view

### Requirement: Command Center Data Is Exposed Via A Dedicated Aggregated View Model
The system SHALL provide a dedicated aggregated read model for the Command Center rather than requiring the UI to derive attention state from multiple unrelated endpoints.

#### Scenario: Aggregated endpoint returns all primary sections
- **WHEN** an authenticated client requests the Command Center data endpoint
- **THEN** the response SHALL include structured sections for attention items, recent critical activity, recovery readiness, and any configured watchlist or upcoming work summaries
- **AND** each section SHALL be individually optional/empty without invalidating the full response

#### Scenario: Recovery readiness states missing verification explicitly
- **GIVEN** recent backups exist but verification data is missing or stale
- **WHEN** the aggregated Command Center response is produced
- **THEN** the recovery-readiness section SHALL distinguish that state from a healthy verified state
- **AND** the UI SHALL be able to render the absence of verification as an explicit caution rather than as silent success
