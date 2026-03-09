# control-plane Specification

## Purpose
TBD - created by archiving change add-in-app-docs. Update Purpose after archive.
## Requirements
### Requirement: Hub Serves Product Documentation Under /docs
The Hub SHALL serve a static product documentation site under `/docs/` without requiring authentication by default.

#### Scenario: /docs redirects and docs pages render
- **WHEN** a client requests `GET /docs`
- **THEN** the Hub redirects to `/docs/`
- **AND WHEN** a client requests `GET /docs/`
- **THEN** the Hub serves the documentation index page

### Requirement: Docs Can Be Served From FS Or Embedded
The Hub SHALL support serving docs either from the filesystem (via `BASTION_DOCS_DIR`) or from embedded assets when built with an `embed-docs` feature.

#### Scenario: Filesystem docs dir override is honored
- **GIVEN** `BASTION_DOCS_DIR` points to a directory containing `index.html`
- **WHEN** a client requests `GET /docs/`
- **THEN** the Hub serves `index.html` from that directory

### Requirement: Node-Scoped WebDAV Browsing API
The system SHALL provide a node-scoped API to list WebDAV directory entries to support selecting WebDAV destination prefixes in the Web UI.

#### Scenario: List a WebDAV directory on a node
- **WHEN** the user requests `GET/POST /api/nodes/<node_id>/webdav/list` with `{ base_url, secret_name, path }`
- **THEN** the API returns the directory entries (files and subdirectories) with basic metadata

### Requirement: Hub Runtime Config Is Persisted In DB
The Hub SHALL persist a runtime config document in the settings table to support UI-driven configuration.

#### Scenario: Saved config survives restarts
- **WHEN** the user saves runtime config via the API
- **AND** the Hub is restarted
- **THEN** the saved config MUST be loadable from the DB

### Requirement: Startup Applies Saved Config With Safe Precedence
On startup, the Hub SHALL apply saved runtime config values only when the corresponding CLI/ENV value is not explicitly set.

#### Scenario: CLI/ENV overrides DB
- **GIVEN** a saved runtime config value exists in DB
- **WHEN** the Hub is started with an explicit CLI or ENV value for that field
- **THEN** the effective runtime config MUST use the CLI/ENV value

#### Scenario: DB applies when CLI/ENV is default
- **GIVEN** a saved runtime config value exists in DB
- **WHEN** the Hub is started without explicitly setting that field via CLI or ENV
- **THEN** the effective runtime config MUST use the DB value

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

### Requirement: HTTPS Required for Non-Loopback Access
When insecure mode is not enabled, the system SHALL reject non-loopback requests unless they are determined to be HTTPS via a trusted reverse proxy.

#### Scenario: Direct HTTP access is rejected
- **WHEN** a non-loopback client accesses the service over plain HTTP (not behind a trusted reverse proxy)
- **THEN** the request is rejected with an error indicating HTTPS is required

### Requirement: Login Throttling
The system SHALL throttle repeated failed login attempts to mitigate brute-force attacks.

#### Scenario: Too many failed attempts triggers lockout
- **WHEN** a client exceeds the allowed number of failed login attempts within a time window
- **THEN** subsequent login attempts are rejected for a cooldown period

### Requirement: Client IP Extraction from Trusted Proxy
The system SHALL determine the effective client IP from `X-Forwarded-For` only when the request originates from a configured trusted proxy, and SHALL otherwise use the direct peer IP.

#### Scenario: Trust X-Forwarded-For only from trusted proxy
- **WHEN** a request includes `X-Forwarded-For` from an untrusted source
- **THEN** the system ignores it for security decisions (e.g., throttling)

### Requirement: Master Key Keypack Export/Import
The system SHALL support exporting the master keyring (`data/master.key`) as a password-encrypted keypack and importing it back to restore the master keyring.

#### Scenario: Keypack export/import round trip
- **WHEN** a user exports a keypack with a password and later imports it with the same password
- **THEN** the restored `data/master.key` can decrypt existing encrypted secrets in SQLite

#### Scenario: Wrong keypack password is rejected
- **WHEN** a user attempts to import a keypack using the wrong password
- **THEN** the import fails without overwriting the existing `data/master.key`

### Requirement: Master Key Rotation
The system SHALL support rotating the master keyring by generating a new active key while retaining old keys for decryption of existing data.

#### Scenario: Existing secrets remain decryptable after rotation
- **WHEN** the master key is rotated
- **THEN** previously stored encrypted secrets remain decryptable

### Requirement: Backup Encryption Key Storage
The system SHALL store backup encryption keys in the encrypted secrets store protected by `data/master.key`.

#### Scenario: Encryption key is created on demand
- **WHEN** a user enables age encryption using key name `K` and the key does not exist
- **THEN** the system creates and stores an encryption key under name `K` and uses it for the run

