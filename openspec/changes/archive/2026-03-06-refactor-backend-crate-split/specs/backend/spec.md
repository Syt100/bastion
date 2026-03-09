## ADDED Requirements

### Requirement: Backend Is Split Into Focused Crates
The backend codebase SHALL be organized into focused crates with clear responsibilities (HTTP, engine/orchestration, storage, backup, targets, notifications, and shared core types).

#### Scenario: Adding a target is isolated
- **WHEN** a new backup target type (e.g., S3) is implemented
- **THEN** the implementation primarily lives in the targets crate and does not require refactoring unrelated HTTP routing code

### Requirement: Backend Crate Dependencies Are Layered
The backend crate dependency graph SHALL be acyclic and SHOULD follow a layered architecture where the HTTP crate depends on the engine layer rather than directly depending on storage/backup/targets internals.

#### Scenario: HTTP does not bypass the engine layer
- **WHEN** an HTTP handler triggers a backup run or restore action
- **THEN** the handler calls into the engine layer rather than reaching into low-level backup/target modules directly

### Requirement: Core Types Remain Lightweight
The shared core crate (`bastion-core`) SHALL remain lightweight and MUST NOT depend on heavy runtime frameworks such as `axum`, `tokio`, `reqwest`, or `sqlx`.

#### Scenario: Core crate stays framework-free
- **WHEN** `bastion-core` is compiled
- **THEN** it does not pull in HTTP/runtime/storage framework dependencies

