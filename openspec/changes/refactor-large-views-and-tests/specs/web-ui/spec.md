## ADDED Requirements

### Requirement: Large Web UI Screens SHALL Separate Orchestration From Presentation
Large Web UI screens/components SHALL extract shared orchestration logic (query sync, async loading, bulk actions, or picker state) into dedicated composables/modules.

#### Scenario: Refactored view keeps existing behavior
- **GIVEN** a large screen with filters, pagination, and action handlers
- **WHEN** orchestration logic is extracted to composables/modules
- **THEN** visible behavior and route/query semantics stay unchanged
- **AND** the screen becomes easier to maintain with smaller focused units

### Requirement: Jobs Modal Flows SHALL Have Direct Regression Coverage
Critical jobs modal flows SHALL include direct component tests for open/submit/error behavior.

#### Scenario: Modal flow changes trigger test failures when behavior regresses
- **GIVEN** jobs modals such as editor/deploy/runs/restore/verify
- **WHEN** open flow, primary action, or API error handling regresses
- **THEN** component-level tests fail and surface the regression
