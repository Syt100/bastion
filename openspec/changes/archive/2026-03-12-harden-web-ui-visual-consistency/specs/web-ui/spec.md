## ADDED Requirements

### Requirement: Foundation Tokens for Visual Consistency
The Web UI SHALL define a small set of globally stable “foundation” tokens (e.g., radii and motion) that are not theme-specific and are used consistently across shared components.

#### Scenario: Foundation tokens control shared primitives
- **GIVEN** the UI renders common primitives (cards, toolbars, panels)
- **WHEN** the foundation tokens are updated
- **THEN** shared primitives update consistently without page-by-page overrides

### Requirement: Muted Text Uses Theme Tokens (Not Opacity)
The Web UI SHALL render secondary/muted text using theme tokens (e.g. `--app-text-muted`) and SHOULD avoid using opacity utilities to communicate semantic “muted” meaning for text.

#### Scenario: Muted text remains consistent across themes
- **WHEN** the user switches between light/dark mode or theme presets
- **THEN** muted text remains readable and visually consistent
- **AND** it does not become too faint or too strong due to stacked opacity

### Requirement: Dividers and Borders Use Theme Tokens
The Web UI SHALL render dividers and subtle borders using theme tokens (e.g. `--app-border`) rather than hard-coded black/white translucency utilities.

#### Scenario: List separators match the active theme
- **GIVEN** the UI renders a list with separators
- **WHEN** the user switches themes
- **THEN** separators and subtle borders continue to match the active surface hierarchy

### Requirement: Standardized Component Recipes
The Web UI SHALL provide and use standardized recipes for common UI patterns, including at minimum:
- Card / inset panel,
- list row,
- list/filter toolbars,
- tags/badges (status vs neutral),
- data tables,
- mono/code blocks and keycap hints.

#### Scenario: Two pages share the same look for the same pattern
- **GIVEN** two pages render the same pattern (e.g. a clickable list row)
- **WHEN** both pages are viewed in light and dark mode
- **THEN** the pattern looks consistent (spacing, radius, divider, hover/pressed, typography)

### Requirement: Guardrails Prevent Non-Token Styling Regressions
The repository SHALL include an automated guardrail check that detects reintroduced non-token styling patterns in `ui/src` (e.g. hard-coded Tailwind “semantic colors” and disallowed arbitrary values) and fails CI when violations are introduced.

#### Scenario: CI blocks a non-token color regression
- **WHEN** a contributor introduces a forbidden non-token styling pattern in `ui/src`
- **THEN** the guardrail check fails
- **AND** the failure message guides the contributor toward the token-based alternative

### Requirement: Developer-Facing UI Style Guide (EN + zh-CN)
The project SHALL document the Web UI visual system in the docs site, including:
- token inventory and meaning,
- approved spacing/typography/radius scales,
- recipes for common patterns,
- and do/don’t examples for consistency.

#### Scenario: Contributor can learn the rules without tribal knowledge
- **GIVEN** a new contributor needs to add or modify a Web UI screen
- **WHEN** they open the developer docs
- **THEN** they can find the UI style guide and apply the documented recipes

### Requirement: Visual Accessibility Remains Intact
The consistency refactor SHALL preserve focus-visible indicators, SHOULD maintain adequate contrast for text (including muted text) in light/dark mode, and SHOULD respect reduced motion preferences.

#### Scenario: Keyboard focus remains clearly visible
- **WHEN** the user navigates the UI using the keyboard
- **THEN** focus-visible styling remains clearly visible and consistent

