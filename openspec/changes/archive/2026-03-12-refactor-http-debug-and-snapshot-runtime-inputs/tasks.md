## 1. Spec

- [x] 1.1 Add proposal/design/spec artifacts for request-scoped debug rendering and explicit snapshot runtime settings
- [x] 1.2 Run `openspec validate refactor-http-debug-and-snapshot-runtime-inputs --strict`

## 2. Implementation

- [x] 2.1 Replace HTTP process-global debug error state with request-scoped render options and response-time gating
- [x] 2.2 Capture filesystem snapshot runtime settings at execution entry points and pass them explicitly into snapshot helpers
- [x] 2.3 Add regression tests for scoped debug rendering and explicit snapshot settings

## 3. Validation

- [x] 3.1 Run targeted Rust tests for `bastion-http`, `bastion-backup`, `bastion-engine`, and `bastion`
- [x] 3.2 Run `scripts/ci.sh`
