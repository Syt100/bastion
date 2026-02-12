## ADDED Requirements

### Requirement: Validation Errors Must Expose Structured Semantics
For user-actionable validation errors, backend responses MUST keep top-level `error` and `message`, and MUST expose machine-readable semantics in `details`.

At minimum, validation responses SHALL support:
- `details.reason` for sub-cause classification under the same `error` code
- `details.field` for the primary field when a single field is involved
- `details.params` for typed parameters needed by UI localization templates

#### Scenario: Single code with multiple meanings is disambiguated by reason
- **WHEN** two validation branches share the same top-level `error` code
- **THEN** each branch includes a distinct `details.reason`
- **AND** frontend clients can distinguish branches without parsing `message`

#### Scenario: Parameterized validation includes machine-readable params
- **WHEN** a validation rule uses thresholds (such as min/max length)
- **THEN** the response includes threshold values in `details.params`
- **AND** the values can be used for localized UI templates

### Requirement: Backend Supports Multi-Field Validation Violations
When one request has multiple field validation failures, backend responses MUST support a structured violations list.

The violations list SHALL support per-item:
- `field`
- `reason`
- optional `params`
- optional human-readable `message`

#### Scenario: Multiple field failures are returned without lossy flattening
- **WHEN** request validation detects more than one field failure
- **THEN** the response includes `details.violations[]`
- **AND** each violation includes at least `field` and `reason`

### Requirement: Agent List Error Transport Must Be Structured
Filesystem/WebDAV browse errors relayed from agents to hub MUST include machine-readable error codes.

For agent-originated list errors:
- the transport SHALL include `error_code`
- the hub SHALL map known agent codes to stable API error codes without relying on message substrings
- message text remains a fallback for unknown legacy cases only

#### Scenario: Filesystem list not-directory maps from structured code
- **WHEN** agent returns a filesystem list failure with a machine-readable not-directory code
- **THEN** hub API responds with stable `error = "not_directory"`
- **AND** this mapping does not depend on `message` text

#### Scenario: Legacy agent message still falls back safely
- **WHEN** an older agent does not provide structured error code
- **THEN** hub still returns a meaningful API error using compatibility fallback
- **AND** newer structured mapping takes precedence when available

### Requirement: Structured Contract Remains Backward Compatible
The backend SHALL keep the existing top-level API error envelope (`error`, `message`, `details`) while introducing structured detail fields.

#### Scenario: Existing clients that only read error/message continue to work
- **WHEN** a client ignores `details.reason` and `details.params`
- **THEN** the client still receives valid `error` and `message`
- **AND** behavior remains backward compatible
