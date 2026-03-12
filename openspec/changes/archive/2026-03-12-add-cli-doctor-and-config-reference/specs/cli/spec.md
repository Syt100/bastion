## ADDED Requirements

### Requirement: CLI Provides A Config Inspection Command
The Bastion CLI SHALL provide a `bastion config` command to inspect the effective Hub configuration.

The output MUST include the effective value and the resolved source for each field:
`cli|env|db|default`.

The command SHOULD support a machine-readable `--json` output mode.

#### Scenario: Operator inspects effective config
- **WHEN** the operator runs `bastion config`
- **THEN** the CLI prints effective config values and sources

#### Scenario: Operator requests JSON output
- **WHEN** the operator runs `bastion config --json`
- **THEN** the CLI prints a JSON document with the same information

### Requirement: CLI Provides A Doctor Command For Common Setup Issues
The Bastion CLI SHALL provide a `bastion doctor` command to run diagnostics for common setup issues.

The command MUST:
- validate the configured data directory is usable
- validate the Hub database can be opened
- validate required static assets are available (embedded or filesystem paths)

The command MUST exit non-zero on failures.

The command SHOULD support a machine-readable `--json` output mode.

#### Scenario: Doctor reports a missing docs directory
- **GIVEN** Bastion is built without embedded docs and the docs directory is missing
- **WHEN** the operator runs `bastion doctor`
- **THEN** the CLI reports a failure with guidance
- **AND** exits non-zero

### Requirement: New CLI Help Text Is Covered By zh-CN Translations
When `zh-CN` help output is enabled, CI MUST fail if the zh-CN translation resource is missing any required key for the CLI command tree.

#### Scenario: Missing translation fails CI
- **GIVEN** a required translation key is missing for `bastion doctor`
- **WHEN** CI runs the repository checks
- **THEN** the CI job fails

