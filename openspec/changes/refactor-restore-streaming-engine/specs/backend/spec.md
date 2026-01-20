---
## ADDED Requirements

### Requirement: Restore Uses a Streaming Engine with Pluggable Sources and Sinks
The backend SHALL implement restore via a streaming restore engine that consumes a pluggable artifact source and writes to a pluggable restore sink, to enable future restore destinations while preserving current restore semantics.

#### Scenario: Restore logic is decoupled from storage backends
- **WHEN** a developer adds a new restore destination backend
- **THEN** they implement a new restore sink without modifying archive parsing logic

### Requirement: Restore Refactor Preserves Existing Behavior
The backend SHALL preserve current restore behavior (selection filtering, conflict policy semantics, and operation events) while refactoring restore internals.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

