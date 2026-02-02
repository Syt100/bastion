## ADDED Requirements

### Requirement: Workspace Centralizes Shared Dependency Versions
The Rust workspace SHALL centralize versions for dependencies used across multiple crates by declaring them in `[workspace.dependencies]`.

#### Scenario: Shared dependency versions are defined in one place
- **WHEN** a shared dependency version is updated
- **THEN** it is updated in the root `Cargo.toml` under `[workspace.dependencies]`
- **AND** all workspace crates consume that version via `workspace = true`

### Requirement: Member Crates Use `workspace = true` For Centralized Dependencies
Workspace member crates MUST reference centralized dependencies using `workspace = true` to avoid per-crate version drift.

#### Scenario: Cargo build uses a consistent version set
- **WHEN** the workspace is built (`cargo test --workspace`)
- **THEN** Cargo resolves consistent dependency versions across all workspace members without per-crate version pinning for centralized dependencies

