## ADDED Requirements

### Requirement: Preset Theme Selection (6 Themes)
The Web UI SHALL provide a preset theme system that allows the user to choose one of **six** curated color schemes.

#### Scenario: User selects a different theme
- **GIVEN** the user opens Settings → Appearance
- **WHEN** the user selects a theme preset
- **THEN** the UI updates immediately (colors, surfaces, and background aurora)
- **AND** the selection persists across page reloads

### Requirement: Default Theme Is Mint Teal (Fresh + Bright)
The Web UI SHALL ship with **Mint Teal** as the default theme, targeting a fresh and bright look:
- light mode uses a mint-tinted page background with clean white content surfaces,
- teal is used as the primary action/selection accent,
- dark mode uses higher-contrast "blacker black / whiter white" surfaces and text (avoiding muddy gray haze).

#### Scenario: Fresh default palette is applied on first load
- **GIVEN** the user has no stored theme preference
- **WHEN** the Web UI loads
- **THEN** Mint Teal is applied automatically
- **AND** the UI remains readable and visually consistent in both light and dark mode

### Requirement: Theme-Specific Background Aurora
Each theme preset SHALL be able to define its own background aurora/gradient layers to reinforce the theme personality, while keeping the solid base background color separate.

#### Scenario: Background aurora does not reduce readability
- **WHEN** a theme is applied
- **THEN** the page background renders as `solid base + subtle aurora layers`
- **AND** text and primary content surfaces remain legible on both desktop and mobile

### Requirement: No Custom Theme Editing (This Iteration)
The Web UI SHALL NOT provide user-defined custom color editing in this iteration; only the preset themes are selectable.

#### Scenario: User cannot enter arbitrary colors
- **WHEN** the user opens Settings → Appearance
- **THEN** the UI offers only predefined theme choices
- **AND** there is no custom color picker or free-form input

### Requirement: Naive UI Theme Overrides Track Active Theme
The Web UI SHALL ensure that Naive UI theme overrides update when the active theme changes and SHALL avoid passing CSS `var(...)` strings into overrides.

#### Scenario: Theme switch updates component palette safely
- **WHEN** the user switches between themes
- **THEN** Naive UI components reflect the new colors immediately
- **AND** no runtime color parsing errors occur due to `var(...)` strings

### Requirement: Browser Theme Color Reflects Active Theme
The Web UI SHALL update `meta[name="theme-color"]` to reflect the active theme:
- in light mode it SHOULD use the theme’s primary accent color,
- in dark mode it SHOULD use the theme’s solid background color.

#### Scenario: Mobile browser chrome matches theme
- **GIVEN** the user is on a mobile browser
- **WHEN** the user toggles light/dark mode or switches theme presets
- **THEN** the browser chrome color updates to match the active theme intent
