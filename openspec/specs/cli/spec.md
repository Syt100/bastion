# cli Specification

## Purpose
TBD - created by archiving change add-cli-help-i18n-docgen. Update Purpose after archive.
## Requirements
### Requirement: CLI Help Output Is Locale-Aware
The Bastion CLI SHALL render `--help` output in the user's preferred language.

The default CLI language MUST be English (`en-US`).

The CLI help locale resolution order MUST be:

1. `BASTION_LANG` (accepted: `en`, `en-US`, `zh`, `zh-CN`)
2. `LC_ALL`, then `LC_MESSAGES`, then `LANG` (any value indicating `zh*` selects Chinese)
3. Default English (`en-US`)

#### Scenario: Default CLI help is English
- **WHEN** the user runs `bastion --help` with no `BASTION_LANG` and no `zh*` system locale
- **THEN** the help output is English

#### Scenario: BASTION_LANG overrides system locale
- **GIVEN** the system locale indicates `zh*`
- **WHEN** the user runs `bastion --help` with `BASTION_LANG=en-US`
- **THEN** the help output is English

#### Scenario: System locale selects Chinese
- **WHEN** the user runs `bastion --help` with `LANG=zh_CN.UTF-8` (or equivalent `LC_*`)
- **THEN** the help output is Chinese (`zh-CN`)

### Requirement: CLI Help Strings Use Stable Translation Keys
For supported locales beyond English, the CLI help strings SHALL be localized via a stable key scheme derived from the CLI command tree.

The key format MUST include:

- Command about text: `<cmd_path>.about`
- Arg help text: `<cmd_path>.arg.<arg_id>.help`
- Arg long help text: `<cmd_path>.arg.<arg_id>.long_help`

#### Scenario: Adding a new flag creates a new required key
- **WHEN** a new CLI flag is introduced with a help text
- **THEN** a corresponding translation key is required for `zh-CN`

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

