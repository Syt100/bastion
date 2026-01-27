## ADDED Requirements

### Requirement: CI Can Build Docs Site For Embedded Builds
The repository CI workflow SHALL be able to build the docs site so that `embed-docs` builds succeed.

#### Scenario: CI builds docs with /docs base
- **WHEN** CI runs the repository checks
- **THEN** it builds the docs site with `DOCS_BASE=/docs/`

