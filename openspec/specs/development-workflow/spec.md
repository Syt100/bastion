# development-workflow Specification

## Purpose
TBD - created by archiving change refactor-backend-crate-split. Update Purpose after archive.
## Requirements
### Requirement: Crate Split Supports Incremental Development
The codebase SHALL maintain a modular crate structure that enables faster incremental development by isolating high-churn code from heavy/low-churn code.

#### Scenario: Routine changes stay localized
- **WHEN** a developer changes an HTTP handler or WebSocket message mapping
- **THEN** the change is localized to the HTTP crate (and its direct dependencies) rather than forcing unrelated backup/target implementation changes

### Requirement: Milestone and Bug Fix Commits
During implementation, the developer SHALL create a Git commit after each milestone feature is completed and after each bug fix is completed.

#### Scenario: Milestone feature commit
- **WHEN** a milestone feature is completed
- **THEN** the changes are committed to Git before starting the next milestone

#### Scenario: Bug fix commit
- **WHEN** a bug fix is completed
- **THEN** the fix is committed to Git before starting unrelated work

### Requirement: Commit Message Format
Commits created for milestones and bug fixes SHALL use a detailed message that summarizes changes by module and lists key modifications.

#### Scenario: Commit message is detailed
- **WHEN** a milestone or bug fix commit is created
- **THEN** the commit message includes:
  - a short title line
  - a `Modules:` section with bullet points grouped by module (e.g., `openspec`, `ui`, `bastion`, `bastion-core`)
  - a `Tests:` section listing relevant commands executed (or `not run` with reason)

