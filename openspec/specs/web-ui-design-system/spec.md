# web-ui-design-system Specification

## Purpose
TBD - created by archiving change add-web-ui-background-styles. Update Purpose after archive.
## Requirements
### Requirement: Background Style Selection
The Web UI MUST allow users to choose a background style independently from the theme preset to reduce visual noise while keeping semantic tokens consistent.

Supported styles:
- `aurora`: themed base color + gradient background (default)
- `solid`: themed base color, no gradient
- `plain`: neutral base color (white/black depending on dark mode), no gradient

#### Scenario: Switching background style
- **WHEN** a user changes the background style in Settings → Appearance
- **THEN** the background updates immediately without requiring a reload
- **AND** the choice persists across sessions

#### Scenario: Plain background uses neutral base
- **WHEN** background style is `plain`
- **THEN** the page base background uses neutral white/black (not theme-tinted)
- **AND** the aurora gradient is disabled

#### Scenario: Plain background neutralizes surfaces and navigation chrome
- **WHEN** background style is `plain`
- **THEN** card/overlay surfaces use neutral colors (not theme-tinted) in both light and dark mode
- **AND** navigation chrome surfaces (e.g., glass sidebar/topbar) use neutral colors in dark mode

#### Scenario: Mobile browser chrome stays neutral in plain mode
- **WHEN** background style is `plain`
- **THEN** `meta[name="theme-color"]` uses the neutral base color so the address bar is not tinted by the theme primary color

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

