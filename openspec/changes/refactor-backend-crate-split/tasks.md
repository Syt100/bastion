## 1. Spec
- [x] 1.1 Add `backend` spec delta for focused crate boundaries and dependency layering
- [x] 1.2 Add `development-workflow` spec delta for the crate-split architecture expectation
- [x] 1.3 Run `openspec validate refactor-backend-crate-split --strict`

## 2. Backend refactor
- [x] 2.1 Add new crates: `bastion-config`, `bastion-storage`, `bastion-targets`, `bastion-backup`, `bastion-notify`, `bastion-engine`, `bastion-http`
- [x] 2.2 Move shared types to `bastion-core` and update imports
- [x] 2.3 Move DB/migrations/repos/secrets/auth to `bastion-storage`
- [x] 2.4 Move targets implementations to `bastion-targets`
- [x] 2.5 Move backup/restore modules to `bastion-backup`
- [x] 2.6 Move notifications delivery to `bastion-notify`
- [x] 2.7 Move scheduler/maintenance/run bus/agent management to `bastion-engine`
- [x] 2.8 Move HTTP router/handlers to `bastion-http` and keep `embed-ui` feature via dependency feature forwarding
- [x] 2.9 Update `bastion` binary wiring to use the new crates

## 3. Validation
- [x] 3.1 Run `cargo fmt`
- [x] 3.2 Run `cargo clippy --workspace -- -D warnings`
- [x] 3.3 Run `cargo test --workspace`
