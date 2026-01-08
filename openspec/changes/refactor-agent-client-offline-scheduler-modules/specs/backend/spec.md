---
## ADDED Requirements

### Requirement: Agent Offline Scheduler Code Is Split Into Focused Submodules
The backend SHALL organize agent offline scheduler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Cron logic changes are localized
- **WHEN** a developer needs to change offline cron scheduling behavior
- **THEN** the change primarily occurs in the cron-loop submodule and does not require edits to worker execution or sink parsing logic

### Requirement: Agent Offline Scheduler Refactor Preserves Behavior
The backend SHALL preserve existing offline scheduler behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

