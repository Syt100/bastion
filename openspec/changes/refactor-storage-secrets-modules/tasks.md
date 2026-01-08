## 1. Spec
- [x] 1.1 Add `backend` spec delta for: storage secrets modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-storage-secrets-modules --strict`

## 2. Backend - Storage secrets modularization
- [x] 2.1 Identify responsibilities and module boundaries (`crypto`, `keyring`, `keypack`, `io`)
- [x] 2.2 Convert `secrets.rs` into a folder module and keep public re-exports stable
- [x] 2.3 Extract SecretsCrypto + EncryptedSecret into `crates/bastion-storage/src/secrets/crypto.rs`
- [x] 2.4 Extract master keyring structures/validation into `crates/bastion-storage/src/secrets/keyring.rs`
- [x] 2.5 Extract keypack export/import into `crates/bastion-storage/src/secrets/keypack.rs`
- [x] 2.6 Extract atomic file helpers into `crates/bastion-storage/src/secrets/io.rs`
- [x] 2.7 Keep tests passing (adjust module paths only; no behavior changes)
- [x] 2.8 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
