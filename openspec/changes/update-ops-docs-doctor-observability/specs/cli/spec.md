## MODIFIED Requirements

### Requirement: CLI Provides A Doctor Command For Common Setup Issues
The `bastion doctor` command SHALL provide actionable diagnostics for common operational issues.

In addition to the existing checks, the command MUST:
- validate the configured bind address is usable (port is free, or Bastion is already listening on it)
- warn when Bastion is configured to bind to a non-loopback address without `--insecure-http` (reverse proxy/TLS expected)

The command MUST exit non-zero on failures.

The command SHOULD continue to support a machine-readable `--json` output mode.

#### Scenario: Doctor warns about non-loopback bind without insecure http
- **GIVEN** the Hub bind address is non-loopback
- **AND** `--insecure-http` is not enabled
- **WHEN** the operator runs `bastion doctor`
- **THEN** the command outputs a warning explaining that a reverse proxy/TLS is required for non-loopback access
