## ADDED Requirements

### Requirement: Data Directory Location
The system SHALL store state in a data directory, defaulting to `<exe_dir>/data`, and SHALL allow overriding the path via the `BASTION_DATA_DIR` environment variable.

#### Scenario: Data directory override
- **WHEN** the service starts with `BASTION_DATA_DIR=/custom/path`
- **THEN** it uses `/custom/path` for the SQLite DB, master key, and caches

### Requirement: Data Directory Must Be Writable
The system SHALL ensure the selected data directory is writable at startup and SHALL fall back to an OS-specific default location if `<exe_dir>/data` is not writable and `BASTION_DATA_DIR` is not set.

#### Scenario: Windows fallback location
- **WHEN** `<exe_dir>/data` is not writable on Windows and `BASTION_DATA_DIR` is not set
- **THEN** the system uses an OS-specific writable default (e.g., under `%ProgramData%`)

### Requirement: Default Bind and Port
The system SHALL listen on `127.0.0.1:9876` by default and SHALL allow configuring the bind address and port, including binding to `0.0.0.0`.

#### Scenario: Local-only default
- **WHEN** the service starts with no bind configuration
- **THEN** it is reachable only on `127.0.0.1:9876`

### Requirement: Reverse Proxy Support
The system SHALL support deployment behind a reverse proxy and SHALL only trust `X-Forwarded-*` headers from configured trusted proxies.

#### Scenario: Reject untrusted forwarded headers
- **WHEN** requests include `X-Forwarded-Proto` from an untrusted source
- **THEN** the system ignores the header for security decisions

### Requirement: Single-User Authentication
The system SHALL provide single-user authentication using a password hashed with Argon2id and SHALL maintain login state using a cookie session stored in SQLite.

#### Scenario: Login creates a session
- **WHEN** a user logs in with correct credentials
- **THEN** the system creates a server-side session record in SQLite and sets an HttpOnly session cookie

### Requirement: CSRF Protection
The system SHALL protect state-changing HTTP requests against CSRF attacks.

#### Scenario: Missing CSRF token is rejected
- **WHEN** an authenticated user submits a state-changing request without a valid CSRF token
- **THEN** the system rejects the request

### Requirement: Insecure Mode is Explicit
The system SHALL require an explicit configuration option to allow insecure HTTP/WS operation for LAN/dev usage and SHALL present persistent warnings in the Web UI when enabled.

#### Scenario: Insecure mode warning
- **WHEN** the system is started with insecure mode enabled
- **THEN** the Web UI shows a persistent warning that tokens and traffic are not protected by TLS
