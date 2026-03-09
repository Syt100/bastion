## ADDED Requirements

### Requirement: UI Tests Reuse Shared Naive-UI Stub Helpers
UI unit tests SHALL reuse shared Naive UI stub helpers for common components to reduce duplication and avoid repeated warning-prone stub behavior.

#### Scenario: Multiple view specs import shared stubs
- **GIVEN** view specs need Naive UI component stubs
- **WHEN** tests are implemented or updated
- **THEN** specs import common stub helpers instead of redefining identical base stubs
- **AND** shared input stubs safely avoid invalid native prop forwarding warnings

### Requirement: Router Meta Configuration Uses Shared Builders for Repeated Blocks
When route meta structures are repeated across multiple routes, router config SHALL use shared helper builders to keep metadata consistent and reduce drift.

#### Scenario: Repeated settings meta uses a shared builder
- **GIVEN** multiple settings routes share title/back-navigation meta patterns
- **WHEN** router meta is defined
- **THEN** shared helper builders generate repeated meta blocks
- **AND** resulting route behavior remains unchanged
