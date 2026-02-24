## ADDED Requirements

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
