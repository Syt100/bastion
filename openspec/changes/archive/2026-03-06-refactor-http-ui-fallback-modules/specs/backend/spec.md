---
## ADDED Requirements

### Requirement: HTTP UI Fallback Code Is Isolated From Router Setup
The backend SHALL isolate UI fallback asset-serving logic into a focused module to improve maintainability, without changing behavior.

#### Scenario: UI cache header changes are localized
- **WHEN** a developer needs to adjust cache-control or ETag behavior for UI assets
- **THEN** the change primarily occurs in the UI fallback module and does not require edits to API routing setup

### Requirement: HTTP UI Fallback Refactor Preserves Behavior
The backend SHALL preserve UI fallback behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

