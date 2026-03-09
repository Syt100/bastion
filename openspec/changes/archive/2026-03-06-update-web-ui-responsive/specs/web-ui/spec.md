## ADDED Requirements

### Requirement: Responsive Navigation and Layout
The Web UI SHALL be responsive and SHALL present exactly one navigation pattern per breakpoint:
- Mobile (`< md`): top bar with hamburger + drawer navigation.
- Desktop (`>= md`): persistent sidebar navigation (no drawer navigation).

#### Scenario: Desktop navigation does not include a drawer
- **WHEN** the viewport is `>= md`
- **THEN** the sidebar navigation is visible and the hamburger/drawer navigation is not shown

#### Scenario: Mobile navigation uses a drawer
- **WHEN** the viewport is `< md`
- **THEN** the sidebar navigation is not shown and navigation is accessible via the hamburger/drawer

### Requirement: Header and Content Alignment
On wide screens, the Web UI header controls SHALL align to the same container baseline as the main page content.

#### Scenario: Header aligns with content on wide screens
- **WHEN** the viewport is wide enough that a max-width container applies
- **THEN** header controls align horizontally with the main content container

### Requirement: Mobile Card Lists for Tabular Pages
For tabular list pages, the Web UI SHALL render a mobile-friendly card list on small screens while keeping tables on desktop.

#### Scenario: Jobs list renders as cards on mobile
- **WHEN** the user views the Jobs page on a mobile viewport (`< md`)
- **THEN** jobs are rendered as cards with primary actions available without horizontal scrolling

### Requirement: Dialog Sizing
Dialogs in the Web UI SHALL be constrained to sensible maximum widths on desktop and SHALL remain usable on mobile.

#### Scenario: Credential editor does not occupy full desktop width
- **WHEN** a user opens a credential editor dialog on a desktop viewport
- **THEN** the dialog width is constrained and does not span the full window width

### Requirement: Brand Mark Icon
The Web UI brand mark SHALL use Ionicons `ShieldCheckmark` (solid) and SHALL not appear visually distorted.

#### Scenario: Brand icon is not visually compressed
- **WHEN** the brand mark is displayed in the header or sidebar
- **THEN** the icon maintains its intended proportions and does not appear squeezed

### Requirement: UI Copy Punctuation Consistency
The Web UI SHALL render subtitles and short helper texts without trailing periods in both `zh-CN` and `en-US`.

#### Scenario: Login subtitle has no trailing punctuation
- **WHEN** the login page subtitle is rendered
- **THEN** it does not end with a trailing period character

