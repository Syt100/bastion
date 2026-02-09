## 1. Spec
- [x] 1.1 Add `backend` spec deltas for proxy/origin/setup/auth hardening
- [x] 1.2 Add `hub-agent` spec deltas for artifact stream authorization binding
- [x] 1.3 Add `ops-docs` spec deltas for reverse proxy safe forwarding guidance
- [x] 1.4 Run `openspec validate harden-agent-stream-and-edge-security --strict`

## 2. Agent Artifact Stream Authorization (P0)
- [x] 2.1 Add task/operation-bound authorization check before hub artifact stream open
- [x] 2.2 Add security logs for denied stream-open attempts
- [x] 2.3 Add regression tests for cross-agent denial and authorized success

## 3. Trusted Proxy and Client IP Hardening (P0)
- [x] 3.1 Replace left-most XFF parsing with trusted-hop right-to-left algorithm
- [x] 3.2 Add/adjust tests for spoofed XFF and multi-hop trusted chains
- [x] 3.3 Update reverse proxy docs with safer forwarding recommendations

## 4. Run Events WS Origin Hardening (P1)
- [x] 4.1 Validate effective scheme+host+port instead of host only
- [x] 4.2 Add regression tests for scheme/port mismatch cases

## 5. Setup/Auth Hardening (P1)
- [x] 5.1 Make setup initialize atomic under concurrent requests
- [x] 5.2 Add backend validation policy for username/password
- [x] 5.3 Add regression tests for race + invalid input

## 6. Maintainability Refactor (P2)
- [x] 6.1 Split `agents/ws` into focused modules while preserving behavior
- [x] 6.2 Replace synchronous filesystem I/O in async hot paths where practical
- [x] 6.3 Keep/extend tests to guard refactor behavior

## 7. Validation
- [x] 7.1 Run `cargo fmt --all`
- [x] 7.2 Run targeted tests for touched crates
- [x] 7.3 Run `bash scripts/ci.sh`
- [x] 7.4 Mark checklist complete
