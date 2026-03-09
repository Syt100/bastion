## ADDED Requirements

### Requirement: Toast-Style Error Messages Use the Shared Error Formatter
For non-form actions and modal workflows that surface errors via toasts, the Web UI SHALL use the shared error formatter so that:
- Known backend `error` codes are localized.
- Unknown codes fall back to backend `message`.

This reduces regressions where UI code only shows a generic “failed” message or loses the backend error code.

#### Scenario: Toast displays localized known error code
- **WHEN** an API call fails and returns a known `error` code
- **THEN** the UI shows the localized message for that code

#### Scenario: Toast falls back to backend message for unknown code
- **WHEN** an API call fails and returns an unknown `error` code with a `message`
- **THEN** the UI shows the backend `message`

### Requirement: UI Captures Request ID and Surfaces It for Internal Errors
The Web UI SHALL capture `X-Request-Id` from API responses and attach it to the thrown API error object.

For 5xx/internal errors, the UI SHOULD surface the Request ID to help users correlate UI failures with server logs.

#### Scenario: Internal error includes a Request ID for troubleshooting
- **WHEN** an API call fails with HTTP 500 `internal_error`
- **AND** `X-Request-Id` is present in the response headers
- **THEN** the UI surfaces the Request ID to the user

