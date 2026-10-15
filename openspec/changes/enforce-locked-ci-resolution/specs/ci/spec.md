## ADDED Requirements

### Requirement: Main CI Validates The Committed Cargo Lockfile
Every dependency-resolving Cargo invocation in the main CI script SHALL use `--locked`.

#### Scenario: Committed lockfile requires normalization
- **WHEN** a main CI Cargo command would rewrite Cargo.lock
- **THEN** the command fails rather than silently replacing the committed dependency graph

#### Scenario: Committed lockfile is consistent
- **WHEN** the main CI script resolves the workspace dependencies
- **THEN** it uses the committed Cargo.lock without rewriting it
