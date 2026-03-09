## ADDED Requirements

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

