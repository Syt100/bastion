## ADDED Requirements

### Requirement: Notifications Sub-Navigation Uses A Single Source Of Truth
The Web UI SHALL derive Notifications settings subpages (mobile list entries and desktop tabs) from a single shared config.

#### Scenario: Adding a new Notifications subpage updates all entry points
- **WHEN** a new Notifications subpage is added to the shared config
- **THEN** it appears in the Notifications index list
- **AND** it appears in the desktop Notifications tab bar

### Requirement: Tests Guard Against Notifications Nav/Router Drift
The Web UI SHALL include unit tests that fail if a configured Notifications subpage does not resolve in the router or has inconsistent metadata.

#### Scenario: Configured subpages always resolve
- **GIVEN** a Notifications subpage in the shared config
- **THEN** `router.resolve(to)` MUST match at least one route record

#### Scenario: Title keys stay consistent
- **GIVEN** a Notifications subpage in the shared config
- **THEN** its router route SHOULD use the same `meta.titleKey`

### Requirement: Language Options Use A Single Source Of Truth
The Web UI SHALL derive language dropdown options from `supportedLocales` via a single shared helper.

#### Scenario: A new locale becomes selectable everywhere
- **WHEN** a locale is added to `supportedLocales`
- **THEN** it is available in language dropdowns across the app

### Requirement: Tests Guard Against Locale Option Drift
The Web UI SHALL include unit tests that fail if a supported locale is missing a label or a Naive UI locale mapping.

#### Scenario: Supported locales are fully mapped
- **GIVEN** a locale in `supportedLocales`
- **THEN** it MUST have a dropdown label
- **AND** it MUST have Naive UI `locale` and `dateLocale` mappings

