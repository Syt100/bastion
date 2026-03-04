## ADDED Requirements

### Requirement: Interactive Icons SHALL Use Shared Size and Tone Semantics
The UI SHALL provide a shared icon wrapper for interactive controls so icon size and semantic tone remain consistent.

#### Scenario: Action icons keep consistent dimensions
- **GIVEN** multiple action controls render icons across list pages
- **WHEN** the controls are displayed together
- **THEN** icons use shared size semantics instead of arbitrary per-page sizing
- **AND** semantic tone mapping is applied consistently

### Requirement: Modal Shells SHALL Reuse a Shared Structural Wrapper
The UI SHALL render critical card-style modals through a shared modal shell wrapper with consistent spacing and scroll behavior.

#### Scenario: Modal content and footer remain stable across pages
- **GIVEN** the user opens modal dialogs on Jobs and Agents pages
- **WHEN** dialogs include long content or loading transitions
- **THEN** body spacing and footer alignment remain consistent
- **AND** modal content area scroll behavior is predictable

### Requirement: Shared Micro-Interaction Rules SHALL Be Applied to Core List Surfaces
Core list surfaces SHALL use shared motion rules for hover/focus/press feedback with reduced-motion fallback.

#### Scenario: Users receive consistent interaction feedback
- **GIVEN** users interact with list rows, cards, and filter trigger controls
- **WHEN** hover/focus/press states occur
- **THEN** interactions use shared duration/easing tokens
- **AND** reduced-motion preference disables non-essential transitions
