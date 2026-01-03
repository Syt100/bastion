## ADDED Requirements

### Requirement: Settings Mobile TopBar Is Stable and Centered
On mobile viewports (`< md`), the Web UI SHALL render a dedicated TopBar for all pages under `/settings/**`.

The TopBar MUST:
- have a fixed height so page content does not shift vertically when navigating between Settings pages,
- reserve fixed left and right regions so the centered title does not shift horizontally,
- center the title text within the TopBar,
- reserve the right-side actions region, but it SHALL be empty in this version.

#### Scenario: Title does not shift when back button appears
- **WHEN** the user navigates from `/settings` to `/settings/storage`
- **THEN** the Settings mobile title remains horizontally centered
- **AND** the overall header height remains unchanged

### Requirement: TopBar Title and Back Target Are Route-Meta Driven
The Settings mobile TopBar SHALL derive its `title` and `back` behavior from route metadata for the current matched route.

The effective TopBar config SHALL be determined by the deepest matched route that provides TopBar metadata.

#### Scenario: Notifications subpage shows subpage title and returns to Notifications index
- **WHEN** the user opens `/settings/notifications/channels` on mobile
- **THEN** the TopBar title displays `Channels` (localized)
- **AND** tapping Back navigates to `/settings/notifications`

### Requirement: No Mobile Subtitle Under Settings Title
On mobile viewports, Settings pages SHALL NOT show the desktop-style subtitle under the Settings title.

Explanatory copy SHOULD be rendered as regular in-page gray helper text where needed.

#### Scenario: Settings subtitle hidden on mobile
- **WHEN** the user opens `/settings` on a mobile viewport
- **THEN** no subtitle is shown under the title area

