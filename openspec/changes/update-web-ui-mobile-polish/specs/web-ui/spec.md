## ADDED Requirements

### Requirement: Centralized Breakpoints and UI Constants
The Web UI SHALL centralize breakpoint definitions and other shared UI constants so responsive behavior and layout sizing remain consistent across the codebase.

#### Scenario: Breakpoint logic is centralized
- **WHEN** responsive behavior requires a breakpoint check
- **THEN** the code uses shared breakpoint constants rather than hard-coded values scattered across files

### Requirement: Safe Menu Navigation
Menu interactions in the Web UI SHALL NOT attempt to navigate to invalid or undefined routes.

#### Scenario: Menu key is invalid
- **WHEN** the menu emits an invalid/undefined key
- **THEN** the UI ignores it and no router navigation is attempted

### Requirement: Mobile Header Overflow Menu
On mobile viewports, the Web UI SHALL present global actions (language selection, theme toggle, logout) via a compact overflow menu so the header does not overflow.

#### Scenario: Mobile header actions do not overflow
- **WHEN** the viewport is `< md`
- **THEN** global actions are accessible without header content overflowing the screen

### Requirement: Mobile-Friendly Wizard Step Indicator
On mobile viewports, multi-step wizards in the Web UI SHALL use a compact step indicator (step x/total + progress bar) to avoid horizontal overflow, while desktop viewports may show the full stepper.

#### Scenario: Jobs wizard steps fit on mobile
- **WHEN** a user opens the Jobs create/edit wizard on a mobile viewport (`< md`)
- **THEN** the step indicator is readable without horizontal scrolling

### Requirement: Beta Label
The Web UI SHALL display a "Beta" label to indicate the UI is a test version.

#### Scenario: Beta tag is visible
- **WHEN** a user views the main navigation chrome (header/sidebar)
- **THEN** a "Beta" tag is displayed

