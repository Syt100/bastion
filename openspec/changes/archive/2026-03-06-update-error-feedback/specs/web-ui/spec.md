## ADDED Requirements

### Requirement: Forms Show Inline Error Reasons
For form-based workflows, the Web UI SHALL show inline error feedback that includes the failure reason, instead of only showing a generic toast message.

#### Scenario: Settings form shows invalid webhook error inline
- **WHEN** the user saves a WeCom bot secret with an invalid webhook URL
- **THEN** the UI shows an inline error message
- **AND** the error is associated with the webhook URL field when backend `details.field` is present

### Requirement: Known Error Codes Are Localized
The Web UI SHALL translate known backend `error` codes into localized (`zh-CN`/`en-US`) user-facing messages.

#### Scenario: Login invalid credentials is localized
- **WHEN** the backend responds with `error = "invalid_credentials"` during login
- **THEN** the UI displays a localized “invalid credentials” message in the current UI language

### Requirement: Fallback to Backend Message When Unknown
If the UI does not recognize a backend `error` code, it SHALL fall back to displaying the backend `message` (when provided) to avoid losing actionable information.

#### Scenario: Unknown backend error falls back to message
- **WHEN** the backend returns an unrecognized `error` code with a `message`
- **THEN** the UI displays that `message` inline for the user

