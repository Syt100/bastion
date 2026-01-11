---
## ADDED Requirements

### Requirement: Cleanup Page Uses Clear Title
The web UI SHALL label the cleanup page as “Incomplete run cleanup” (and a localized equivalent).

#### Scenario: Chinese locale shows a clear title
- **GIVEN** the UI locale is `zh-CN`
- **WHEN** the user opens the cleanup page
- **THEN** the page title is “不完整运行清理”

### Requirement: UI Explains Cleanup Task Statuses
The web UI SHALL provide an in-page help dialog that explains the meaning of each cleanup task status.

#### Scenario: User opens the status help dialog
- **WHEN** the user clicks the “?” help button on the cleanup page
- **THEN** the UI shows short explanations for `queued`, `running`, `retrying`, `blocked`, `done`, `ignored`, and `abandoned`

