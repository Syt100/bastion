## ADDED Requirements

### Requirement: CLI Reference Docs Are Auto-Generated
The repository SHALL provide an automated way to generate a CLI reference page from the actual CLI definition.

The generated reference MUST include English and Chinese variants.

#### Scenario: Doc generator produces CLI reference pages
- **WHEN** the doc generator is run in write mode
- **THEN** it writes an English CLI reference page under `docs/user/reference/`
- **AND** it writes a Chinese CLI reference page under `docs/zh/user/reference/`

### Requirement: CI Fails On Missing CLI Translations
CI MUST fail when the CLI help translation resource is missing any required `zh-CN` key derived from the CLI definition.

#### Scenario: Missing translation key fails CI
- **GIVEN** a required translation key is missing from the `zh-CN` translation map
- **WHEN** CI runs the repository checks
- **THEN** the CI job fails

### Requirement: CI Fails On Stale Generated Reference Docs
CI MUST fail when the generated CLI reference pages differ from the committed outputs.

#### Scenario: Generated output drift fails CI
- **GIVEN** the CLI definition changed
- **WHEN** CI runs the doc generator in check mode
- **THEN** the CI job fails until the generated docs are updated and committed

