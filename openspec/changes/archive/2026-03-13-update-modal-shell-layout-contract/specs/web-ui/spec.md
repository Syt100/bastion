## ADDED Requirements

### Requirement: Shared Modal Shell SHALL Enforce Container-vs-Body Layout Boundaries
The web UI SHALL enforce a shared modal layout contract where viewport-bounding size constraints are applied to the modal container layer, while body scrolling and internal flow are handled by the modal body layer.

#### Scenario: Long-form dialog remains viewport-bounded
- **GIVEN** a dialog with long form content (for example, task create/edit)
- **WHEN** the dialog opens on desktop or mobile
- **THEN** the modal container height remains bounded by viewport-safe limits
- **AND** the user scrolls dialog content inside the modal body instead of the page growing beyond intended modal bounds

#### Scenario: Plain body mode does not bypass overflow safety
- **GIVEN** a dialog using `scrollBody=false`
- **WHEN** content height exceeds available body space
- **THEN** body layout still respects bounded height and overflow rules defined by the shared modal contract
- **AND** footer actions remain reachable without layout breakage

### Requirement: Modal Layout Regressions SHALL Be Covered by Unit Tests
The web UI SHALL include unit tests for shared modal layout contract behavior so height/overflow regressions are detected before merge.

#### Scenario: Contract tests cover container and body responsibilities
- **GIVEN** the shared modal shell and a representative long-form dialog
- **WHEN** unit tests run in CI
- **THEN** tests verify container-level sizing inputs and body-level scrolling behavior are applied to the expected layers
- **AND** regressions that move viewport constraints into incorrect layers fail tests
