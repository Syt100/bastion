---
## ADDED Requirements

### Requirement: Picker Modals Support Keyboard Shortcuts
The web UI picker modals SHALL support keyboard shortcuts for common navigation actions.

#### Scenario: Navigate with keyboard
- **GIVEN** a picker modal is open
- **WHEN** the user presses `Backspace` while not typing in an input
- **THEN** the picker navigates to the parent directory/prefix
- **WHEN** the user presses `Ctrl/Cmd+L`
- **THEN** the path/prefix editor receives focus
- **WHEN** the user presses `Esc`
- **THEN** the modal closes

### Requirement: Picker Modals Provide Accessible Labels
The web UI picker modals SHALL provide accessible labels and predictable focus order for icon-only controls.

#### Scenario: Icon-only actions have an accessible label
- **GIVEN** a picker modal renders icon-only controls (e.g., refresh, up)
- **WHEN** the controls are focused
- **THEN** they expose an accessible name via `aria-label` (and/or `title` as a fallback)
