## ADDED Requirements

### Requirement: Effective Client IP Uses Trusted-Hop Forwarded Chain Parsing
The backend SHALL derive effective client IP behind trusted proxies by walking forwarded hops from right to left, removing trusted proxy hops, and selecting the first untrusted hop.

#### Scenario: Spoofed left-most forwarded IP does not bypass throttling key
- **GIVEN** a trusted reverse proxy forwards a request with attacker-controlled left-most `X-Forwarded-For`
- **WHEN** the backend derives effective client IP
- **THEN** it does not blindly trust the left-most entry
- **AND** login throttling keys remain bound to the effective untrusted client hop

#### Scenario: Invalid forwarded chain falls back safely
- **WHEN** forwarded headers are malformed or cannot be parsed
- **THEN** the backend falls back to peer IP

### Requirement: Run Events WebSocket Origin Validation Compares Full Origin Tuple
The backend SHALL validate run-events WebSocket origin using effective scheme, host, and port.

#### Scenario: Host matches but port differs
- **WHEN** request host matches but origin port differs from effective request port
- **THEN** the backend rejects the connection as invalid origin

#### Scenario: Host matches but scheme differs
- **WHEN** request host matches but origin scheme differs from effective request scheme
- **THEN** the backend rejects the connection as invalid origin

### Requirement: Setup Initialization Is Atomic
The backend SHALL enforce atomic first-user initialization so concurrent setup requests cannot initialize more than once.

#### Scenario: Concurrent setup requests race
- **WHEN** multiple setup requests are processed concurrently
- **THEN** at most one request succeeds
- **AND** all others receive a conflict-style error

### Requirement: Backend Enforces Authentication Input Policy
The backend SHALL enforce input policy for auth/setup credentials regardless of client behavior.

#### Scenario: Blank username is rejected
- **WHEN** setup or auth receives an empty or whitespace-only username
- **THEN** the backend returns a validation error

#### Scenario: Too-short password is rejected
- **WHEN** setup receives a password below minimum policy length
- **THEN** the backend returns a validation error
