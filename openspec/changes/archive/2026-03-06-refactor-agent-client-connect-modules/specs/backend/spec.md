---
## ADDED Requirements

### Requirement: Agent Connect Code Is Split Into Focused Submodules
The backend SHALL organize agent websocket connect code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Message handling changes are localized
- **WHEN** a developer needs to adjust how a specific hub message is processed
- **THEN** the change primarily occurs in the message-handlers submodule and does not require edits to handshake or heartbeat logic

### Requirement: Agent Connect Refactor Preserves Behavior
The backend SHALL preserve existing agent websocket connect behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

