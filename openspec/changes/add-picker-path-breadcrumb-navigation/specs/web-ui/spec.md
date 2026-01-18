## MODIFIED Requirements

### Requirement: Picker Path Bars Support Breadcrumb Navigation
The web UI SHALL render picker path/prefix inputs using a shared path bar that supports:
- icon-only up/refresh actions,
- an editable input mode for typing/pasting a path, and
- a breadcrumb mode that renders the current path/prefix as clickable segments for fast navigation.

When the path/prefix contains many segments, the breadcrumb mode SHALL collapse middle segments into an `…` control:
- on desktop, `…` SHALL open a popover listing the hidden segments,
- on mobile, `…` SHALL open a bottom drawer listing the hidden segments.

#### Scenario: Jump to a parent directory by clicking a breadcrumb segment
- **GIVEN** the filesystem picker is open at a deep path
- **WHEN** the user clicks a breadcrumb segment representing a parent directory
- **THEN** the picker navigates to that directory and refreshes the listing

#### Scenario: Navigate via collapsed segments menu
- **GIVEN** the picker is open at a deep path where middle segments are collapsed into `…`
- **WHEN** the user opens `…` and selects a hidden segment
- **THEN** the picker navigates to that directory and refreshes the listing

