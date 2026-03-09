# ops-docs Specification

## Purpose
TBD - created by archiving change harden-agent-stream-and-edge-security. Update Purpose after archive.
## Requirements
### Requirement: Reverse Proxy Guidance Documents Safe Forwarded Header Practices
Operations documentation SHALL provide reverse proxy guidance that avoids trusting attacker-injectable forwarded header values.

#### Scenario: Nginx forwarding guidance avoids left-most trust pitfalls
- **WHEN** operators follow Bastion reverse-proxy documentation for Nginx
- **THEN** the recommended `X-Forwarded-For` handling is compatible with backend trusted-hop parsing
- **AND** the docs explain the trust assumptions for forwarded headers

