## ADDED Requirements

### Requirement: Standard Error Response Includes Optional Details
The backend SHALL return errors using a standard JSON body containing:
- `error` (machine-readable error code)
- `message` (human-readable message)
- `details` (optional structured diagnostics)

`details` SHALL NOT include secrets or sensitive values.

#### Scenario: Backend returns a structured error with details
- **WHEN** a request fails due to invalid user input
- **THEN** the backend responds with a 4xx status
- **AND** the response JSON includes `error` and `message`
- **AND** the response JSON includes `details` with safe structured fields (e.g. `field`)

### Requirement: Validation Failures Use 4xx and Stable Error Codes
The backend SHALL surface user-input validation failures as 4xx responses with stable `error` codes and actionable messages, rather than returning HTTP 500.

#### Scenario: Invalid WeCom webhook URL returns 400 with field details
- **WHEN** the user saves a WeCom bot secret with a webhook URL that cannot be parsed as a URL
- **THEN** the backend responds with HTTP 400
- **AND** the response includes `error = "invalid_webhook_url"`
- **AND** `details.field = "webhook_url"`

#### Scenario: Invalid SMTP mailbox returns 400 with field details
- **WHEN** the user saves an SMTP secret with an invalid `from` email address
- **THEN** the backend responds with HTTP 400
- **AND** the response includes `error = "invalid_from"`
- **AND** `details.field = "from"`

### Requirement: Rate Limit Responses Include Retry-After Details
When the backend rate-limits login attempts, it SHALL include machine-readable retry information in `details`.

#### Scenario: Login rate limit includes retry-after seconds
- **WHEN** a client is rate-limited during login
- **THEN** the backend responds with HTTP 429
- **AND** the response includes `error = "rate_limited"`
- **AND** `details.retry_after_seconds` is present

