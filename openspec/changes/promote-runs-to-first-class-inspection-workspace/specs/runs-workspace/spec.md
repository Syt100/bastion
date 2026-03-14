## ADDED Requirements

### Requirement: Runs Is A First-Class Top-Level Workspace
The Web UI SHALL provide `Runs` as a first-class top-level surface rather than only exposing runs from within job history pages.

#### Scenario: Operator opens the global Runs index
- **WHEN** an authenticated operator opens the `Runs` surface
- **THEN** the page SHALL list runs independently of the Jobs workspace
- **AND** the page SHALL support navigating to dedicated run detail pages

#### Scenario: Runs uses stable canonical routes
- **WHEN** an operator navigates between the Runs index and run detail pages
- **THEN** the route family SHALL remain rooted under canonical top-level `/runs` paths
- **AND** any originating job or scope context SHALL be preserved separately from the stable run identity

### Requirement: Runs Index Supports Cross-Context Filtering
The Runs index SHALL support cross-job and cross-node filtering suitable for triage workflows.

#### Scenario: Operator filters failed runs across jobs
- **WHEN** the operator filters the Runs index by failed status
- **THEN** the page SHALL return failed runs regardless of owning job
- **AND** the results SHALL remain filterable by additional job, node, or time criteria

#### Scenario: Scope-aware filters preserve explicit run identity
- **GIVEN** the operator opens a deep-linked run detail and later returns to the Runs index
- **WHEN** the Runs index restores list context
- **THEN** the filter model SHALL preserve explicit scope and filter state separately from the selected run identity

#### Scenario: Runs index supports all-nodes triage
- **GIVEN** the operator selects `all` scope
- **WHEN** the Runs index is queried
- **THEN** the resulting list SHALL span hub and agent-backed runs
- **AND** the page SHALL remain further filterable by job, run kind, status, and time window without collapsing back to a single node model

### Requirement: Run Detail Uses A Dedicated Page
Run inspection SHALL use a dedicated run detail page rather than depending on a modal-first interaction model.

#### Scenario: Run detail is directly addressable
- **WHEN** the operator opens a run from the Runs index, Jobs workspace, or Command Center
- **THEN** the UI SHALL navigate to a dedicated run detail route
- **AND** the run SHALL remain inspectable via direct refresh or shared link without needing its owning page as context

#### Scenario: Old job-scoped run links can be normalized during migration
- **GIVEN** the operator opens an old job-scoped run path that is still covered by a temporary client-side alias
- **WHEN** the route is resolved during migration
- **THEN** the app SHALL normalize to the canonical run detail route
- **AND** the resulting page SHALL preserve originating job or scope context needed for return navigation

### Requirement: Run Detail Prioritizes Root Cause And Next Actions
The first visible portion of run detail SHALL prioritize concise diagnosis and next-step actions over verbose payloads.

#### Scenario: Failed run shows root cause before raw event details
- **GIVEN** a run has terminal `failed` status
- **WHEN** the dedicated run detail page renders
- **THEN** the first visible summary SHALL include normalized failure information and immediate actions
- **AND** the operator SHALL NOT need to open raw JSON or a detail dialog to see the primary failure summary

#### Scenario: Mobile run detail keeps actions above long diagnostics
- **WHEN** a mobile operator opens run detail
- **THEN** the first viewport SHALL keep primary actions and root-cause summary available before long event lists or raw payload sections
- **AND** secondary context such as related operations or artifacts SHALL render after the first-screen diagnostic answer

### Requirement: Run Detail Exposes Run-Centric Actions
The run detail page SHALL expose run-centric actions directly from the run context.

#### Scenario: Eligible restore or verify action is available from run detail
- **GIVEN** a run is eligible for restore or verify workflows
- **WHEN** the operator opens the run detail page
- **THEN** the page SHALL expose those actions directly from the run context
- **AND** the operator SHALL NOT need to navigate back to the owning job first

#### Scenario: Cancellation remains available from run detail
- **GIVEN** a run is `queued` or `running`
- **WHEN** the operator opens the run detail page
- **THEN** the page SHALL expose the cancel action consistent with cancellation lifecycle rules

#### Scenario: Run action availability is driven by explicit capabilities
- **WHEN** the UI renders run detail actions
- **THEN** restore, verify, cancel, and related actions SHALL be driven by explicit run/detail capability metadata or equivalent authoritative rules
- **AND** the UI SHALL NOT rely on ad hoc heuristics spread across multiple components

### Requirement: Mobile Run Detail Uses A Task-First Order
Mobile run detail SHALL present actionable diagnosis before verbose secondary detail.

#### Scenario: Mobile failed run summary appears before the event console
- **WHEN** a mobile operator opens a failed run detail page
- **THEN** the first viewport SHALL prioritize status, root cause, stage, duration, and immediate next actions
- **AND** the full event console SHALL appear after that summary rather than before it
