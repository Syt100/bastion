---
## ADDED Requirements

### Requirement: Backup Restore Entrypoint Is Kept Focused
The backend SHALL keep the restore module entrypoint focused by moving tests and implementation details into dedicated modules, without changing behavior.

#### Scenario: Restore types remain accessible
- **WHEN** a caller uses `bastion_backup::restore::ConflictPolicy` or `bastion_backup::restore::RestoreSelection`
- **THEN** the types remain available with the same paths and semantics after refactoring

### Requirement: Restore Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing restore behavior while refactoring restore module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

