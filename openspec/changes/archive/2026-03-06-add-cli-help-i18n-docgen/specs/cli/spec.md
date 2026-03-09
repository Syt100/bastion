## ADDED Requirements

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

