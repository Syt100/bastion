## ADDED Requirements

### Requirement: HTTP Internal Error Rendering Is Request-Scoped
Backend HTTP internal-error rendering SHALL derive debug-detail exposure from request-scoped runtime options associated with the current router state rather than from process-global mutable state.

#### Scenario: Concurrent routers use different debug policies
- **GIVEN** two backend routers exist in the same process with different `debug_errors` settings
- **WHEN** each router renders an internal error response
- **THEN** each response SHALL reflect only its own router's debug policy
- **AND** one router's setting SHALL NOT leak into the other router's response

#### Scenario: Internal error response hides debug details when disabled
- **GIVEN** a backend request produces an internal error
- **WHEN** the current request-scoped render policy has `debug_errors` disabled
- **THEN** the response SHALL omit debug diagnostic details

### Requirement: Filesystem Snapshot Resolution Uses Explicit Runtime Settings
Backend filesystem snapshot preparation SHALL derive runtime snapshot settings from explicit caller-provided input captured at execution entry points rather than by reading environment variables inside lower-level resolution helpers.

#### Scenario: Execution entry point passes captured snapshot settings
- **GIVEN** a filesystem backup execution path needs to prepare a source snapshot
- **WHEN** runtime snapshot settings are needed
- **THEN** the execution entry point SHALL capture those settings once and pass them into snapshot resolution explicitly
- **AND** the lower-level snapshot helper SHALL NOT read process environment directly

#### Scenario: Snapshot tests remain explicit-input driven
- **GIVEN** snapshot preparation behavior is covered by backend tests
- **WHEN** tests validate enablement and allowlist behavior
- **THEN** the tests SHALL pass explicit snapshot settings into the helper
- **AND** they SHALL NOT depend on shared process-global environment state
