## 1. Spec
- [x] 1.1 Draft proposal/design for runtime log filter environment isolation
- [x] 1.2 Add `backend` spec delta for explicit log-filter env inputs and parallel-safe tests
- [x] 1.3 Run `openspec validate refactor-runtime-log-filter-env-isolation --strict`

## 2. Implementation
- [x] 2.1 Capture runtime env input at `crates/bastion` entry points and thread it through runtime config resolution
- [x] 2.2 Refactor log-filter resolution/logging construction to avoid direct `RUST_LOG` reads in helper functions
- [x] 2.3 Replace env-mutation tests with explicit-input tests and remove obsolete env-guard helpers

## 3. Validation
- [x] 3.1 Run `cargo test -p bastion --bin bastion resolve_`
- [x] 3.2 Run `cargo test -p bastion --bin bastion`
