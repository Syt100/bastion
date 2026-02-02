## 1. Spec
- [x] 1.1 Add `backend` spec delta for RunEventsBus mutex-poisoning hardening
- [x] 1.2 Run `openspec validate harden-run-events-bus-locking --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - RunEventsBus hardening
- [ ] 2.1 Replace `expect(...)` on mutex lock with poison-tolerant locking
- [ ] 2.2 Add a regression test that proves publish/subscribe still works after poisoning
- [ ] 2.3 Run `cargo test --workspace`
- [ ] 2.4 Commit RunEventsBus hardening changes (detailed message)
