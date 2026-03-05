## ADDED Requirements

### Requirement: Node-Scoped Route Paths SHALL Use Shared Builders
Node-scoped navigation paths SHALL be constructed and parsed through shared route helpers to prevent drift in encoding and suffix handling.

#### Scenario: Node id and suffix normalization stay consistent
- **GIVEN** a node id that may contain special characters
- **WHEN** UI code builds jobs/settings paths or switches node context
- **THEN** node route helpers are used for encoding and suffix normalization
- **AND** resulting paths remain consistent across pages

### Requirement: Clipboard Feedback SHALL Reuse Shared Behavior
Views that expose copy actions SHALL use a shared copy+feedback primitive for success/error toasts.

#### Scenario: Copy action feedback is consistent across views
- **GIVEN** the user clicks copy actions on agents/settings pages
- **WHEN** clipboard write succeeds or fails
- **THEN** the same success/error feedback behavior is used

### Requirement: Icon-Only Actions SHALL Provide Accessible Labels
Icon-only action controls SHALL provide explicit accessible labels through a shared component contract.

#### Scenario: Help buttons remain readable by assistive technologies
- **GIVEN** an icon-only help button in desktop or mobile layouts
- **WHEN** the control is rendered
- **THEN** it exposes a non-empty accessible label

### Requirement: Picker Modal Wrappers SHALL Align With Shared Modal Shell
Picker modal wrappers and picker confirmation card dialogs SHALL align with shared modal shell structure while preserving existing flows.

#### Scenario: Picker confirm modal keeps behavior after shell alignment
- **GIVEN** the user opens picker current-directory confirmation
- **WHEN** modal shell wrappers are aligned
- **THEN** title/content/footer actions remain unchanged
- **AND** body/footer spacing and structure follow shared shell conventions
