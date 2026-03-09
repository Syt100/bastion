## ADDED Requirements

### Requirement: Fallback Error Classification Avoids `internal_error` for Common Root Causes
When a request fails due to a common, user-actionable root cause, the backend SHALL classify it into a stable 4xx/403 error code rather than returning HTTP 500 `internal_error`.

The fallback classification SHALL cover at least:
- IO permission denied -> HTTP 403 `permission_denied`
- IO path not found -> HTTP 404 `path_not_found`
- DB row not found -> HTTP 404 `not_found`

#### Scenario: Permission denied is returned as 403 with a stable code
- **WHEN** a request fails due to `std::io::ErrorKind::PermissionDenied`
- **THEN** the backend responds with HTTP 403
- **AND** the response includes `error = "permission_denied"`

#### Scenario: Not found is returned as 404 with a stable code
- **WHEN** a request fails due to `std::io::ErrorKind::NotFound`
- **THEN** the backend responds with HTTP 404
- **AND** the response includes `error = "path_not_found"`

#### Scenario: DB row not found is returned as 404 with a stable code
- **WHEN** a request fails due to `sqlx::Error::RowNotFound`
- **THEN** the backend responds with HTTP 404
- **AND** the response includes `error = "not_found"`

### Requirement: Debug Error Details Are Off by Default
The backend SHALL support an opt-in “debug error details” mode for troubleshooting.

When debug error details are disabled (default), the backend SHALL NOT include internal diagnostic information in responses.

When enabled, the backend MAY include safe diagnostics in `details.debug` for HTTP 500 `internal_error` responses only.

#### Scenario: Default mode does not expose internal diagnostics
- **WHEN** debug error details are disabled
- **AND** a request fails and results in HTTP 500 `internal_error`
- **THEN** the response does not include `details.debug`

#### Scenario: Debug mode includes internal diagnostics for internal_error only
- **WHEN** debug error details are enabled
- **AND** a request fails and results in HTTP 500 `internal_error`
- **THEN** the response MAY include `details.debug`

