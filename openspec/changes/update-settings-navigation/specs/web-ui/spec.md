## ADDED Requirements

### Requirement: Desktop Settings Sidebar Submenu
On desktop viewports (`>= md`), the Web UI SHALL present Settings sub-navigation as a sidebar submenu under a `Settings` parent item.

The `Settings` parent item SHALL only expand/collapse and MUST NOT navigate to a route when clicked.
The submenu SHALL include at least:
- Overview (`/settings`)
- Storage (`/settings/storage`)
- Notifications (`/settings/notifications`)

#### Scenario: Clicking Settings only expands
- **WHEN** the user clicks the `Settings` parent menu item on desktop
- **THEN** the Settings submenu expands/collapses
- **AND** the current route does not change

#### Scenario: Settings route highlights the correct submenu
- **WHEN** the current route is `/settings/notifications/queue`
- **THEN** the sidebar highlights the `Notifications` submenu item
- **AND** the Settings submenu is expanded

### Requirement: Settings Overview Page
The Web UI SHALL provide a Settings overview page at `/settings` that lists settings areas and navigates to their routes.

On mobile, the Settings overview SHOULD use an app-like list layout.

#### Scenario: Settings overview links to Storage
- **WHEN** the user taps `Storage` in Settings overview
- **THEN** the UI navigates to `/settings/storage`

### Requirement: Notifications Index Page and Mobile List-First Navigation
The Web UI SHALL provide a Notifications index page at `/settings/notifications` that lists Notifications subpages:
- Channels
- Destinations
- Templates
- Queue

On mobile viewports (`< md`), Notifications navigation MUST be list-first:
users enter the index list first, then navigate into a subpage.

On desktop, Notifications subpages MAY provide lightweight in-module navigation (e.g. tabs), but it MUST remain route-based so refresh/back/forward work correctly.

#### Scenario: Mobile enters Notifications via index list
- **WHEN** the viewport is `< md` and the user opens `/settings/notifications`
- **THEN** the UI displays a list of Notifications subpages

