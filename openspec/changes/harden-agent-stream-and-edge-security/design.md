# Design: Harden agent stream authorization and edge security

## Scope
- Agent artifact stream authorization in Hub WS handling.
- Trusted proxy client IP derivation and login throttling inputs.
- Run events WebSocket origin policy.
- Setup initialization + auth input constraints.
- `agents/ws` maintainability refactor to reduce future security regressions.

## Goals
- Ensure a connected agent can only open artifact streams that are explicitly tied to its own pending/running task context.
- Ensure reverse-proxy deployments cannot bypass login throttling by spoofing left-most forwarded IP values.
- Ensure browser-originated WS requests are validated against full origin tuple (scheme, host, port).
- Ensure setup and auth reject unsafe input at backend boundaries, regardless of UI behavior.

## Non-goals
- Protocol version changes.
- Broad auth model redesign (RBAC/multi-user permissions).
- UI/UX redesign.

## Architecture decisions

### 1) Artifact stream authorization binding
- Add an authorization gate before `open_hub_artifact_stream` that validates:
  - `op_id` exists as an open agent task for the current `agent_id`.
  - task payload type is compatible with stream use.
  - task/run linkage matches the requested `run_id`.
- Keep authorization check in a small dedicated helper module with unit tests.
- Emit structured security logs for denied requests.

### 2) Trusted proxy client IP derivation
- Replace left-most XFF parsing with right-to-left chain walk:
  - build candidate chain from `X-Forwarded-For` + peer address.
  - skip trusted proxy hops from the right.
  - first untrusted hop becomes effective client IP.
  - fallback to peer IP on parse errors or empty chain.
- Keep behavior deterministic and testable without network dependencies.

### 3) WebSocket origin strictness
- Parse origin URL and compare effective `scheme`, `host`, `port` against request-effective values.
- For trusted proxy peers, derive effective external origin from forwarded headers; otherwise use local request host/proto.

### 4) Setup/auth hardening
- Atomic setup initialize implemented with DB transaction semantics and uniqueness guard.
- Backend-side validation for username/password:
  - trim and reject empty username.
  - minimum password length and non-empty guard.
- Reuse same validation in setup/login paths where applicable.

### 5) `agents/ws` refactor
- Split into:
  - message dispatch loop
  - artifact stream authorization/open/pull
  - run/operation event handling
  - helper utilities
- Keep public behavior stable while shrinking per-file cognitive load.

## Testing strategy
- Add regression tests for:
  - cross-agent stream open denial
  - authorized stream open success path
  - forwarded chain spoofing resistance
  - origin mismatch by scheme/port
  - concurrent setup initialize race
  - username/password backend validation
