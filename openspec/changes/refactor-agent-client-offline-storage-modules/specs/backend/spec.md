---
## ADDED Requirements

### Requirement: Offline Storage Code Is Split Into Focused Submodules
The backend SHALL organize agent offline storage code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Writer changes are localized
- **WHEN** a developer needs to adjust how offline run events are appended
- **THEN** the change primarily occurs in the writer/IO submodules and does not require edits to on-disk type definitions

### Requirement: Offline Storage Refactor Preserves Behavior
The backend SHALL preserve existing offline storage behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

