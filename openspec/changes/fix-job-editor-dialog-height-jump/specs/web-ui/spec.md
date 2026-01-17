## MODIFIED Requirements

### Requirement: Job Editor Modal Has a Stable Layout on Desktop
The web UI SHALL render the job create/edit modal with a stable shell height on desktop-sized screens. Switching steps and showing/hiding validation feedback SHALL NOT cause the modal footer (action buttons) to shift vertically in the viewport; the modal body SHALL scroll instead.

#### Scenario: Switching steps does not move the footer actions
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user switches between job editor steps with different content heights
- **THEN** the modal footer action buttons remain in a stable position
- **AND** the modal body becomes scrollable when the step content exceeds the available space

#### Scenario: Validation feedback does not change modal height
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** inline validation errors appear or disappear within a step
- **THEN** the modal footer action buttons remain in a stable position

### Requirement: Filesystem / Archive Browser Modals Have a Stable Layout on Desktop
The web UI SHALL render filesystem/archive browser modals with a stable shell height on desktop-sized screens. Variable content (selection summaries, inline warnings/errors, async loading states) SHOULD NOT shift the modal footer; the modal body SHOULD scroll instead.

#### Scenario: Large selections do not expand the modal and push the footer
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user selects many files/directories in a browser modal
- **THEN** the selection summary remains compact (single-line with ellipsis and `+N` when needed)
- **AND** the modal footer action buttons remain in a stable position

