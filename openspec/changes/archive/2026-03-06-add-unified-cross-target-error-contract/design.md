## Context
The project already has multiple error-related mechanisms:
- HTTP API structured errors (`error/message/details`) for request-response flows.
- Driver-level error kinds (`config/auth/network/io/unknown`).
- Run-event diagnostics with per-module custom field keys.

This creates drift across layers and makes future protocol adapters (for example SFTP or cloud-drive APIs) harder to integrate consistently.

## Goals
- Define one canonical error envelope for backend emitted diagnostics.
- Keep the envelope transport-agnostic, with protocol-specific extensions.
- Ensure retry logic consumes structured semantics instead of message matching.
- Ensure UI renders localized, actionable diagnostics from keys and params.
- Keep rollout backward-compatible.

## Non-Goals
- Implement every future target adapter in this change.
- Remove all legacy fields in the first migration phase.

## Canonical Error Envelope (V1)
Proposed envelope payload (embedded in run events and related maintenance events):

```json
{
  "schema_version": "1.0",
  "code": "target.auth.invalid_credentials",
  "kind": "auth",
  "stage": "upload",
  "origin": {
    "layer": "target",
    "component": "webdav",
    "op": "put_part"
  },
  "retriable": {
    "value": false,
    "reason": "auth",
    "retry_after_sec": null
  },
  "hint": {
    "key": "errors.hint.target.auth_invalid",
    "params": {}
  },
  "message": {
    "key": "errors.msg.target.auth_failed",
    "params": {}
  },
  "transport": {
    "protocol": "http"
  },
  "context": {},
  "debug": {
    "request_id": "req-123"
  }
}
```

### Field Rules
- Required: `schema_version`, `code`, `kind`, `retriable.value`, `hint.key`, `message.key`, `transport.protocol`.
- Optional: protocol-specific transport fields, `context`, `debug`.
- `http_status` becomes HTTP-only and MUST NOT be reused for non-HTTP protocols.

## Protocol Extension Rules
- `transport.protocol = "http"`:
  - Optional: `status_code`, `status_text`, `retry_after_sec`, `provider_request_id`.
- `transport.protocol = "sftp"`:
  - Optional: `provider_code` (for example `SSH_FX_PERMISSION_DENIED`), `disconnect_code`.
- `transport.protocol = "drive_api"`:
  - Optional: `provider`, `provider_code`, `provider_request_id`.
- `transport.protocol = "file"`:
  - Optional: `io_kind`, `os_error_code`.

## Async and Partial Failure Extensions
For providers that run async operations or batch semantics:
- `context.operation`:
  - `operation_id`, `status`, `poll_after_sec`.
- `context.partial_failures[]`:
  - Each item includes `resource_id|path`, `code`, `kind`, optional `transport`.

## Retry Semantics
- Retry policy consumes `retriable.value` and optional `retriable.retry_after_sec`.
- `retriable.reason` is mandatory when `value=true` and should be one of stable values (`rate_limited`, `timeout`, `upstream_unavailable`, `transient_network`).

## Localization Strategy
- Backend emits `hint.key` and `message.key` with params.
- UI resolves localized strings using i18n dictionaries.
- If key is missing, UI falls back to:
  1) default locale key,
  2) legacy plain message,
  3) generic localized fallback.

## Compatibility and Rollout
Phase 1:
- Emit canonical envelope plus legacy fields in parallel.

Phase 2:
- UI reads envelope first; legacy fields used only for fallback.

Phase 3:
- Remove deprecated legacy fields after compatibility window and telemetry confirmation.

## Target Capability Matrix (Design Input)
| target_type | protocol | async_operation | partial_failure | required_transport_fields |
| --- | --- | --- | --- | --- |
| local_dir | file | false | optional | `protocol`, optional `io_kind` |
| webdav | http | false | optional | `protocol`, optional `status_code` |
| sftp | sftp | false | optional | `protocol`, optional `provider_code` |
| s3 | http | false | true | `protocol`, optional `provider_code`, optional `status_code` |
| drive_api | drive_api | true | true | `protocol`, optional `provider`, optional `provider_code` |

## Risks / Trade-offs
- Short-term payload size increases because envelope and legacy fields coexist.
- Migration touches multiple modules; regression risk needs contract tests.
- i18n key coverage must be enforced to avoid user-facing key leakage.

## Open Questions
- Exact compatibility window for legacy field removal.
- Whether to enforce strict enum for `code` families at compile time or via lint/test gates.
