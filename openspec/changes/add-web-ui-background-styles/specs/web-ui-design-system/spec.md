## ADDED Requirements

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
