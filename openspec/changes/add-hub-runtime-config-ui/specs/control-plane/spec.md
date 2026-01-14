## ADDED Requirements

### Requirement: Hub Runtime Config Is Persisted In DB
The Hub SHALL persist a runtime config document in the settings table to support UI-driven configuration.

#### Scenario: Saved config survives restarts
- **WHEN** the user saves runtime config via the API
- **AND** the Hub is restarted
- **THEN** the saved config MUST be loadable from the DB

### Requirement: Startup Applies Saved Config With Safe Precedence
On startup, the Hub SHALL apply saved runtime config values only when the corresponding CLI/ENV value is not explicitly set.

#### Scenario: CLI/ENV overrides DB
- **GIVEN** a saved runtime config value exists in DB
- **WHEN** the Hub is started with an explicit CLI or ENV value for that field
- **THEN** the effective runtime config MUST use the CLI/ENV value

#### Scenario: DB applies when CLI/ENV is default
- **GIVEN** a saved runtime config value exists in DB
- **WHEN** the Hub is started without explicitly setting that field via CLI or ENV
- **THEN** the effective runtime config MUST use the DB value

