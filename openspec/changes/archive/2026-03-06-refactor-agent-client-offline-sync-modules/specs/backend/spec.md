---
## ADDED Requirements

### Requirement: Offline Sync Code Is Split Into Focused Submodules
The backend SHALL organize agent offline sync code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Event loading changes are localized
- **WHEN** a developer needs to adjust how offline run events are parsed from `events.jsonl`
- **THEN** the change primarily occurs in the events-loader submodule and does not require edits to HTTP ingest logic

### Requirement: Offline Sync Refactor Preserves Behavior
The backend SHALL preserve existing offline sync behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

