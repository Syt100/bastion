## ADDED Requirements

### Requirement: Public Base URL Is Configurable
The control plane SHALL support an explicit public base URL used for operator-facing command and link generation.

#### Scenario: Public base URL is stored in runtime configuration
- **WHEN** an operator saves a public base URL through the supported runtime-configuration mechanism
- **THEN** the control plane SHALL persist that value as part of the runtime configuration model

#### Scenario: Stored public base URL is normalized
- **WHEN** the control plane accepts a valid public base URL value
- **THEN** it SHALL preserve any configured path prefix needed for reverse-proxy deployments
- **AND** it SHALL normalize trivial formatting differences such as a trailing slash before exposing the effective value

### Requirement: Effective Public Base URL Respects Existing Precedence Rules
The effective public base URL SHALL respect the same safe precedence model used by other persisted runtime configuration.

#### Scenario: Explicit runtime override wins over persisted value
- **GIVEN** a persisted public base URL exists
- **WHEN** the control plane is started with an explicit higher-precedence configuration source for that field
- **THEN** the effective public base URL SHALL use the explicit override instead of the persisted value

#### Scenario: Unset value remains explicitly absent
- **GIVEN** neither a persisted value nor a higher-precedence override exists
- **WHEN** authenticated clients request public metadata
- **THEN** the control plane SHALL expose the public base URL as absent rather than synthesizing one from request origin implicitly

### Requirement: Effective Public Base URL Is Exposed To Authenticated UI Clients
The control plane SHALL expose the effective public base URL to authenticated UI clients that need to generate operator-facing commands or links.

#### Scenario: Authenticated client reads effective public base URL
- **WHEN** an authenticated UI client requests runtime/public metadata for operator-facing link generation
- **THEN** the response SHALL include the effective public base URL when configured
- **AND** the response SHALL make absence explicit when no value is configured

#### Scenario: Lightweight public metadata can be consumed outside System pages
- **WHEN** an authenticated UI surface such as Fleet needs operator-facing URL metadata
- **THEN** the control plane SHALL expose a lightweight authenticated metadata contract suitable for that purpose
- **AND** the client SHALL NOT be required to fetch the full runtime-configuration document merely to render onboarding commands
