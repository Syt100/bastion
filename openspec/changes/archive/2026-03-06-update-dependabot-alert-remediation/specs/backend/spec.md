## ADDED Requirements

### Requirement: Open Dependabot Alerts MUST Be Remediated with Verified Dependency Constraints
Repository dependency updates MUST remediate active Dependabot alerts using manifest/lockfile changes that are validated in CI.

#### Scenario: Rust lockfile contains vulnerable crate patch level
- **WHEN** Rust dependency advisories affect versions in `Cargo.lock`
- **THEN** workspace manifests and lockfiles are updated to patched versions or constrained to avoid vulnerable transitive paths
- **AND** Rust build/test checks continue to pass

#### Scenario: npm transitive vulnerability is reported
- **WHEN** npm alerts are raised for transitive packages in UI/docs lockfiles
- **THEN** manifests apply explicit override constraints to patched versions when needed
- **AND** lockfiles are regenerated and validated with existing build/test quality gates

### Requirement: Dependency Surface MUST Exclude Unused High-Risk Feature Paths
Workspace dependency features MUST disable unused database/runtime stacks that increase vulnerability exposure.

#### Scenario: SQLx default feature set includes unused drivers
- **WHEN** runtime only requires SQLite paths
- **THEN** workspace dependency configuration disables SQLx default features and enables only required features in member crates
- **AND** lockfile no longer carries unused vulnerable crypto/database subgraphs introduced solely by defaults
