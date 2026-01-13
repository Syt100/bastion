## ADDED Requirements

### Requirement: About Page Shows Hub And UI Build Info
The Web UI SHALL provide an authenticated About page that shows Hub and UI version and build time.

#### Scenario: About page is behind authentication
- **WHEN** an unauthenticated user navigates to the About page
- **THEN** the user is redirected to login

#### Scenario: About page shows build info
- **GIVEN** an authenticated user
- **WHEN** they open Settings -> About
- **THEN** the page shows Hub version + build time
- **AND** shows UI version + build time
