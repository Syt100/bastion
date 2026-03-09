## ADDED Requirements

### Requirement: Settings Page Uses an Internal Submenu
The Web UI SHALL keep a single Settings entry in the main navigation and SHALL provide an internal Settings submenu to organize settings into multiple pages.

#### Scenario: Settings has subpages
- **WHEN** a user opens Settings
- **THEN** the UI presents a submenu to navigate to settings subpages

### Requirement: Notifications Settings UI (Channels/Destinations/Templates/Queue)
The Web UI SHALL provide a Notifications settings area with subpages for Channels, Destinations, Templates, and Queue.

On desktop viewports, the Notifications area MAY provide tabs for subpages, but it MUST remain route-based so refresh/back/forward work correctly.
On mobile viewports, the Notifications subpage navigation SHALL use a compact selector (e.g. segmented/select) and SHOULD avoid wide tables by using card lists.

#### Scenario: Desktop uses tabs but remains route-based
- **WHEN** a user switches from Destinations to Queue
- **THEN** the URL updates to the corresponding sub-route
- **AND** refreshing the browser keeps the user on that subpage

#### Scenario: Mobile navigation does not overflow
- **WHEN** the viewport is `< md`
- **THEN** notifications navigation is usable without horizontal overflow

### Requirement: Inline Validation Errors for Forms
For settings and notifications forms, the UI SHALL display errors inline in the form when possible (field-level), and SHALL still show a toast for unexpected failures.

#### Scenario: Invalid webhook URL shows field error
- **WHEN** a user saves a destination with an invalid webhook URL
- **THEN** the webhook URL form item shows an inline validation message

