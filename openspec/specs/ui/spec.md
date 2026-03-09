# ui Specification

## Purpose
TBD - created by archiving change update-security-and-queue-stability. Update Purpose after archive.
## Requirements
### Requirement: Locale Switching Is Last-Write-Wins
The UI locale switch workflow SHALL apply the latest user-selected locale even when previous locale bundles are still loading asynchronously.

#### Scenario: User toggles locale rapidly
- **WHEN** multiple locale switch requests overlap in flight
- **THEN** only the most recent locale selection is applied to i18n state

### Requirement: Dashboard Desktop Prefetch Is Viewport-Aware
The dashboard SHALL avoid desktop-only table prefetch work on non-desktop viewports.

#### Scenario: Mobile dashboard mount
- **WHEN** the dashboard mounts under a non-desktop viewport
- **THEN** desktop table prefetch is skipped

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

