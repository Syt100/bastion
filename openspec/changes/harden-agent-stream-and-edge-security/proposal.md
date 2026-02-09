# Change: Harden agent stream authorization and edge security

## Why
- The current agent artifact stream open path authorizes by agent key only, but does not bind every stream request to a task/operation that belongs to that agent.
- Client IP extraction behind reverse proxies trusts the left-most `X-Forwarded-For` entry, which can be attacker-controlled in common proxy setups.
- Run events WebSocket origin validation compares only host and ignores scheme/port.
- Setup and authentication input handling should be hardened to avoid race conditions and weak input acceptance.
- The `agents/ws` module has grown large enough that security-critical behavior is harder to audit and evolve safely.

## What Changes
- Add explicit task/operation-bound authorization for hub-served artifact stream open requests from agents.
- Harden trusted-proxy client IP resolution using a right-to-left trusted-hop algorithm and align reverse proxy docs accordingly.
- Tighten run events WebSocket same-origin validation to compare effective scheme/host/port.
- Make setup initialization atomic and enforce backend-side auth input policy for username/password.
- Refactor `agents/ws` into focused submodules and reduce synchronous filesystem I/O in async hot paths.
- Add targeted regression tests for authorization, proxy/IP parsing, origin checks, and setup/auth hardening.

## Impact
- Affected areas: `bastion-http`, `bastion-storage`, `bastion`, `docs`
- Security posture: stronger cross-agent isolation and edge trust handling
- Compatibility: no protocol version bump; stricter validation may reject previously accepted invalid/misconfigured requests
