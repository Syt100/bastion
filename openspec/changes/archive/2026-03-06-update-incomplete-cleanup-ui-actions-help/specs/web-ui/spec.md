---
## ADDED Requirements

### Requirement: UI Explains Cleanup Page Actions
The web UI SHALL provide an in-page help dialog that explains the effect of each cleanup page action button.

#### Scenario: User opens the help dialog and sees action explanations
- **GIVEN** the user is on the incomplete run cleanup page
- **WHEN** the user clicks the “?” help button
- **THEN** the UI shows short explanations for “更多”, “立即重试”, “忽略”, and “取消忽略”
