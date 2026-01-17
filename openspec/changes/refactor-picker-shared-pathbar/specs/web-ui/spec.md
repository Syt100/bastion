## MODIFIED Requirements

### Requirement: Picker Modals Reuse a Shared Path Bar UI
The web UI SHALL implement picker modals that include a path/prefix input using a shared path bar component so styling and UX behavior remain consistent across pickers.

The shared path bar SHALL:
- render up/refresh actions as compact icon-only buttons within the input prefix area,
- apply consistent spacing and softened icon styling, and
- provide a `focus()` API so modals can autofocus the input on open.

#### Scenario: Filesystem picker and run-entries picker share the same path bar style
- **GIVEN** the user opens the filesystem picker and the run entries picker
- **WHEN** the user views the path/prefix input area
- **THEN** both pickers present the same icon-only up/refresh actions and spacing
- **AND** both inputs can be autofocused on open via the shared path bar behavior

