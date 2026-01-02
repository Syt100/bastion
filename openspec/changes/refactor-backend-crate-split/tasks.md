## 1. Spec
- [x] 1.1 Add `backend` spec delta for focused crate boundaries and dependency layering
- [x] 1.2 Add `development-workflow` spec delta for the crate-split architecture expectation
- [x] 1.3 Run `openspec validate refactor-backend-crate-split --strict`

## 2. Backend refactor
- [ ] 2.1 Add new crates: `bastion-config`, `bastion-storage`, `bastion-targets`, `bastion-backup`, `bastion-notify`, `bastion-engine`, `bastion-http`
- [ ] 2.2 Move shared types to `bastion-core` and update imports
- [ ] 2.3 Move DB/migrations/repos/secrets/auth to `bastion-storage`
- [ ] 2.4 Move targets implementations to `bastion-targets`
- [ ] 2.5 Move backup/restore modules to `bastion-backup`
- [ ] 2.6 Move notifications delivery to `bastion-notify`
- [ ] 2.7 Move scheduler/maintenance/run bus/agent management to `bastion-engine`
- [ ] 2.8 Move HTTP router/handlers to `bastion-http` and keep `embed-ui` feature via dependency feature forwarding
- [ ] 2.9 Update `bastion` binary wiring to use the new crates

## 3. Validation
- [ ] 3.1 Run `cargo fmt`
- [ ] 3.2 Run `cargo clippy --workspace -- -D warnings`
- [ ] 3.3 Run `cargo test --workspace`
