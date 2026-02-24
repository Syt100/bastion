## ADDED Requirements

### Requirement: UI Locale Messages Load On Demand
The UI SHALL lazy-load locale message bundles and only load the selected startup locale before application mount.

#### Scenario: Browser starts in zh locale
- **WHEN** the initial locale resolves to `zh-CN`
- **THEN** the app loads `zh-CN` messages for first render without eagerly loading `en-US` messages in the same startup path

### Requirement: Locale Preference Behavior Is Preserved
The UI SHALL preserve existing locale resolution and persistence precedence (local storage, cookie, browser fallback) after lazy-loading migration.

#### Scenario: Stored locale exists
- **WHEN** a valid locale preference already exists in storage
- **THEN** the app initializes and persists using that locale without changing user-visible language behavior
