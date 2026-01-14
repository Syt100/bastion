## ADDED Requirements

### Requirement: Settings Navigation Uses A Single Source Of Truth
The Web UI SHALL derive Settings navigation entries (overview list and sidebar submenu) from a single shared config.

#### Scenario: New settings section appears in both places
- **WHEN** a new Settings section is added to the shared Settings nav config
- **THEN** it is visible in the Settings overview list
- **AND** it is visible in the desktop Settings sidebar submenu

### Requirement: Tests Guard Against Overview/Submenu Drift
The Web UI SHALL include unit tests that fail if a Settings entry is visible in the overview list but missing from the sidebar submenu.

#### Scenario: Overview items are always included in submenu
- **GIVEN** a Settings nav entry is configured to show in the overview list
- **THEN** it MUST also be configured to show in the sidebar submenu
