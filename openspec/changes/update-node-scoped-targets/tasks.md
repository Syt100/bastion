## 1. Spec
- [x] 1.1 Add spec deltas for node-scoped targets/credentials and UI scoping behavior
- [x] 1.2 Run `openspec validate update-node-scoped-targets --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend + Agent Protocol
- [x] 2.1 Add node-scoped target/credential storage model (Hub DB)
- [x] 2.2 Add APIs to CRUD node-scoped targets/credentials
- [ ] 2.3 Add protocol support to sync node-scoped targets/credentials to Agents (for later offline execution)
- [x] 2.4 Add/adjust tests for schema and APIs

## 3. Web UI
- [x] 3.1 Node context Storage/Targets UI shows only node-scoped targets
- [x] 3.2 Job editor enforces node-scoped target selection
- [x] 3.3 Add/adjust unit tests for UI scoping

## 4. Validation
- [x] 4.1 Run backend checks (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] 4.2 Run UI checks (`npm run lint --prefix ui`, `npm test --prefix ui`, `npm run build --prefix ui`)

## 5. Commits
- [x] 5.1 Commit backend changes (detailed message)
- [x] 5.2 Commit UI changes (detailed message)
