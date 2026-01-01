## 1. Spec
- [x] 1.1 Add `backend` spec delta for: clippy cleanliness and CI lint gate
- [x] 1.2 Run `openspec validate update-backend-clippy-cleanup --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - Clippy cleanup
- [ ] 2.1 Replace manual `Default` impls with `#[derive(Default)]` where applicable
- [ ] 2.2 Reduce large enum variants via indirection (boxing) where safe
- [ ] 2.3 Fix `collapsible_if` warnings using let-chains (keep behavior identical)
- [ ] 2.4 Fix remaining lints (`clamp`, `is_multiple_of`, redundant closure, etc.)
- [ ] 2.5 Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`
- [ ] 2.6 Commit backend clippy cleanup changes (detailed message)

## 3. CI
- [ ] 3.1 Update CI scripts to run clippy with `-D warnings`
- [ ] 3.2 Commit CI lint gate changes (detailed message)
