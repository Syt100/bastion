## ADDED Requirements

### Requirement: Agents Management Dialogs SHALL Reuse Shared Modal Shell
Agents management dialogs SHALL reuse the shared modal shell component for consistent body spacing, footer actions, and scroll containment.

#### Scenario: Agents labels and bulk dialogs follow shared modal structure
- **GIVEN** the user opens labels, bulk sync, or bulk labels dialogs on the Agents page
- **WHEN** dialog content is rendered and actions are shown in the footer
- **THEN** the dialogs use the shared modal shell wrapper
- **AND** existing form behavior and submit/cancel semantics remain unchanged
