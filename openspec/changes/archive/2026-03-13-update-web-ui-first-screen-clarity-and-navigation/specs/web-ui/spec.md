## ADDED Requirements

### Requirement: Global Shell SHALL Emphasize Primary Content And Unified Actions
The web UI SHALL present navigation chrome and global actions so that primary page content remains visually dominant and desktop/mobile global actions are grouped consistently.

#### Scenario: Desktop shell keeps content primary while grouping global actions
- **GIVEN** the user is on any authenticated desktop page
- **WHEN** the shell renders navigation and topbar controls
- **THEN** the content area remains the dominant visual surface
- **AND** global actions are grouped together instead of competing with page-level actions

#### Scenario: Mobile shell exposes navigation and global actions in one coherent system
- **GIVEN** the user is on any authenticated mobile page
- **WHEN** they open global navigation
- **THEN** they can access navigation, node context, and global actions without hunting across separate menus

### Requirement: Dashboard SHALL Prioritize Actionable First-Screen Information
The dashboard SHALL place high-priority health signals and recent activity ahead of lower-priority analytical context.

#### Scenario: Users see urgent status before secondary analytics
- **GIVEN** the dashboard contains health, run, and trend information
- **WHEN** the page loads on the first screen
- **THEN** actionable status cards and recent activity appear before trend-only context
- **AND** the user can navigate directly to the relevant management surface from those areas

### Requirement: Jobs Workspace SHALL Reduce Redundant Top-Level Layout Decisions
The Jobs workspace SHALL minimize redundant top-level layout choices so users can enter the primary workflow without first interpreting multiple overlapping display modes.

#### Scenario: Users enter the jobs workflow through a clear primary mode
- **GIVEN** the user opens the Jobs workspace
- **WHEN** the top-level toolbar is shown
- **THEN** the page presents a clear primary workflow mode
- **AND** secondary display choices do not require interpreting multiple overlapping groups of toggles

### Requirement: Empty And Auth States SHALL Provide Guided Next Steps
The web UI SHALL turn first-run empty states and the login screen into guided surfaces with clear next actions and product context.

#### Scenario: Agents empty state guides first-time setup
- **GIVEN** there are no connected agents
- **WHEN** the Agents page renders
- **THEN** the empty state explains the setup flow in concise steps
- **AND** the primary setup action is visually obvious

#### Scenario: Login page communicates context and help affordances
- **GIVEN** the user is on the login page
- **WHEN** the page renders
- **THEN** the UI communicates what Bastion manages and offers supporting guidance beyond the credential form

### Requirement: Authenticated Pages SHALL Expose Main Landmark Semantics
Authenticated pages SHALL expose a clear main content landmark to improve navigation for assistive technologies.

#### Scenario: Main content region is discoverable to assistive technology
- **GIVEN** the user navigates an authenticated page with a screen reader or accessibility audit
- **WHEN** the page structure is evaluated
- **THEN** the primary content area is exposed as a main landmark
