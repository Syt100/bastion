## ADDED Requirements

### Requirement: Workspace Avoids `tokio/full`
The Rust workspace MUST NOT enable `tokio`'s `full` feature and MUST instead explicitly declare only the Tokio feature flags required by the codebase.

#### Scenario: CI rejects `tokio/full` in any workspace crate
- **GIVEN** a workspace member declares `tokio` with `features` containing `"full"`
- **WHEN** CI runs the repository checks
- **THEN** the CI job fails with guidance to keep Tokio features minimal

#### Scenario: Workspace builds with minimal Tokio feature flags
- **WHEN** the backend workspace is built and tested (`cargo test --workspace`)
- **THEN** the build succeeds without relying on `tokio/full`

