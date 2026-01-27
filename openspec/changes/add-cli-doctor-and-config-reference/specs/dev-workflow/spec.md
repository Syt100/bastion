## ADDED Requirements

### Requirement: Config Reference Docs Are Auto-Generated
The repository SHALL provide an automated way to generate configuration/environment reference documentation derived from the actual CLI/env definitions.

The generated reference MUST include English and Chinese variants.

#### Scenario: Doc generator produces config reference pages
- **WHEN** the doc generator is run in write mode
- **THEN** it writes an English config reference page under `docs/user/reference/`
- **AND** it writes a Chinese config reference page under `docs/zh/user/reference/`

### Requirement: CI Fails On Stale Generated Config Reference Docs
CI MUST fail when the generated config reference pages differ from the committed outputs.

#### Scenario: Generated output drift fails CI
- **GIVEN** the CLI/env definition changed
- **WHEN** CI runs the doc generator in check mode
- **THEN** the CI job fails until the generated docs are updated and committed

