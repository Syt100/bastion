## ADDED Requirements

### Requirement: Fleet Combines Health Summary With Onboarding
The Web UI SHALL provide a `Fleet` surface that combines node health and onboarding workflows in one coherent area.

#### Scenario: Fleet renders health summary above the list
- **WHEN** an operator opens the Fleet surface
- **THEN** the page SHALL render fleet-level health summary signals such as online/offline or degraded states before or alongside the fleet list
- **AND** onboarding actions SHALL remain reachable from the same surface

#### Scenario: Empty fleet still renders onboarding rail
- **GIVEN** there are no enrolled agents
- **WHEN** the operator opens the Fleet surface
- **THEN** the page SHALL render an onboarding-oriented empty state
- **AND** the empty state SHALL still behave as part of the Fleet surface rather than as a disconnected placeholder

#### Scenario: Fleet uses canonical top-level routes
- **WHEN** the operator navigates between the fleet collection and an individual fleet member
- **THEN** the route family SHALL remain rooted under canonical `/fleet` paths
- **AND** any temporary `/agents` alias used during migration SHALL normalize without changing the underlying agent identity

### Requirement: Fleet List Shows Operationally Meaningful Agent Summary
The Fleet list SHALL show operationally meaningful status for each agent.

#### Scenario: Agent row surfaces health and sync state
- **WHEN** an agent row is rendered in Fleet
- **THEN** the row SHALL expose agent identity, health status, config-sync or drift state, and recent activity signals relevant to fleet operation
- **AND** the row SHALL offer direct navigation to the corresponding Fleet detail or related workflows

#### Scenario: Fleet list is driven by aggregated list data
- **WHEN** the Fleet list is rendered
- **THEN** the data contract SHALL provide enough per-agent summary state to avoid one detail request per row
- **AND** summary counts shown above the list SHALL be derived from the same aggregated response

### Requirement: Fleet Provides Dedicated Agent Detail Pages
The Web UI SHALL provide a dedicated detail view for each fleet member.

#### Scenario: Operator opens a Fleet detail page
- **WHEN** the operator opens a specific agent from the Fleet list
- **THEN** the UI SHALL navigate to a dedicated Fleet detail page
- **AND** the page SHALL summarize the agent's health, sync state, recent operations, and links to related Jobs or storage context

#### Scenario: Fleet detail exposes capability-driven actions
- **WHEN** the operator opens a Fleet detail page
- **THEN** the page SHALL expose actions such as rotate key, revoke, or sync-now according to explicit capability metadata or equivalent authoritative rules
- **AND** unavailable actions SHALL not be inferred only from UI-local heuristics

### Requirement: Generated Onboarding Commands Use Effective Public Base URL
The system SHALL generate operator-facing enrollment commands from the effective configured public base URL rather than assuming browser origin.

#### Scenario: Fleet empty-state command uses configured public URL
- **GIVEN** the control plane has an effective public base URL
- **WHEN** the Fleet onboarding rail or empty state renders an enrollment command
- **THEN** the generated command SHALL use that effective public base URL
- **AND** it SHALL NOT default to the current browser origin when the configured value is present

#### Scenario: Effective public URL is absent
- **GIVEN** no public base URL is configured
- **WHEN** the Fleet surface renders an onboarding command
- **THEN** the UI SHALL render an explicit unresolved or setup-required state instead of silently using browser origin as a canonical value
- **AND** the Fleet surface SHALL guide the operator to configure the public base URL before copying a production onboarding command

### Requirement: Fleet Uses Aggregated Fleet View Models
The system SHALL provide aggregated Fleet list and detail view models suitable for control-console rendering.

#### Scenario: Fleet list view model includes health and pending work
- **WHEN** the UI requests Fleet list data
- **THEN** the response SHALL include enough summary information to render agent health, sync/drift summary, and pending or recent operational context without issuing one request per agent

#### Scenario: Fleet detail view model includes sync and related-job context
- **WHEN** the UI requests Fleet detail data for one agent
- **THEN** the response SHALL include sync-state detail, recent activity, and related-job context needed by the page
- **AND** the UI SHALL not need to assemble those sections from unrelated ad hoc endpoints before rendering the first screen
