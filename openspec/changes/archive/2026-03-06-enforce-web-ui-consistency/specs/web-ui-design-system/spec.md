## ADDED Requirements

### Requirement: Token-Driven Semantic Colors
The Web UI MUST use `--app-*` design tokens (CSS variables) for semantic colors so that light/dark mode and theme presets remain visually correct.

#### Scenario: Implementing muted text
- **WHEN** a contributor needs secondary / help / description text
- **THEN** they use `app-text-muted` (or `color: var(--app-text-muted)`) instead of opacity hacks or Tailwind palette colors

#### Scenario: Implementing status colors
- **WHEN** a contributor needs to render an info/success/warning/danger state
- **THEN** they use semantic components (e.g., `n-alert`, `n-tag`) or token colors (e.g., `var(--app-danger)`) rather than hard-coded colors

### Requirement: Consistent Card Elevation
All “content cards” MUST use the shared `app-card` elevation and MUST NOT rely on default card borders for visual separation.

#### Scenario: Rendering a card in a view
- **WHEN** a view renders a content container using `n-card` with `class="app-card"`
- **THEN** it sets `:bordered="false"` so the appearance is consistent across pages

### Requirement: Documented Visual Rules
The project MUST provide contributor-facing documentation for the Web UI design system rules and recipes.

#### Scenario: Finding the style guide
- **WHEN** a contributor opens the Developer Docs
- **THEN** they can navigate to the Web UI style guide from the docs sidebar and the dev docs index

