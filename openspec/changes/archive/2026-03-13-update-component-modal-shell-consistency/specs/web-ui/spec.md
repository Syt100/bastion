## ADDED Requirements

### Requirement: Reusable Dialog Components SHALL Reuse Shared Modal Shell
Reusable Jobs and Run dialog components SHALL render through the shared modal shell to keep modal body/footers and optional slot behavior consistent.

#### Scenario: Component-level dialogs use the shared shell without behavior regressions
- **GIVEN** the user opens reusable dialogs such as job editor/deploy/runs/restore/verify and run-event details/events
- **WHEN** dialog content, actions, and optional header slots are rendered
- **THEN** the dialog components use the shared modal shell wrapper
- **AND** existing submit/cancel flows, emitted events, and scroll behavior remain unchanged
