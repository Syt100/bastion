## ADDED Requirements

### Requirement: System Is A Low-Frequency Administrative Surface
The Web UI SHALL provide `System` as the home for low-frequency administrative configuration and product metadata.

#### Scenario: System groups runtime, maintenance, appearance, and about
- **WHEN** an operator opens the System surface
- **THEN** the page SHALL provide entry points for runtime configuration, maintenance, appearance, and about
- **AND** those areas SHALL be clearly separated from daily operational workflows such as Jobs, Runs, Fleet, and Integrations

#### Scenario: Old settings root can be normalized to System during migration
- **GIVEN** the operator opens the old Settings root or a low-frequency administrative child route that is still covered by a temporary client-side alias
- **WHEN** the route is resolved during migration
- **THEN** the app SHALL normalize to the canonical System route family
- **AND** the operator SHALL not need to infer whether the destination belongs under Integrations or System from old navigation labels

### Requirement: System Pages Communicate Operational Impact
System pages SHALL communicate restart requirements, destructive impact, or low-frequency administrative context clearly.

#### Scenario: Restart-required configuration is labeled
- **WHEN** a runtime configuration field requires restart or delayed application semantics
- **THEN** the System UI SHALL label that requirement explicitly
- **AND** the operator SHALL be able to understand that impact before saving

#### Scenario: Dangerous maintenance action shows impact scope
- **WHEN** a maintenance action can delete, revoke, or otherwise mutate important product state
- **THEN** the UI SHALL describe the action's impact scope before confirmation

#### Scenario: Runtime page includes public-base-url impact
- **WHEN** the operator views or edits runtime configuration related to public operator-facing URLs
- **THEN** the System runtime page SHALL explain that this value affects generated enrollment commands and other operator-facing links
- **AND** the page SHALL distinguish configured canonical URL from missing configuration clearly

### Requirement: Daily Operational Navigation Does Not Depend On System
Daily operational workflows SHALL not require navigating through the System surface.

#### Scenario: Storage and notification management live outside System
- **WHEN** an operator needs to manage storage or notification integrations
- **THEN** the navigation path SHALL lead through Integrations rather than System

#### Scenario: Fleet onboarding does not require opening System first
- **WHEN** an operator needs to enroll a new agent
- **THEN** the primary workflow SHALL begin in Fleet
- **AND** System runtime configuration SHALL appear only as supporting context when the public base URL has not been configured
