# Change: Refactor storage secrets module structure

## Why
`crates/bastion-storage/src/secrets.rs` is a large module mixing concerns (master keyring IO/validation, encryption/decryption helpers, key rotation, keypack export/import, and tests). Splitting it into focused submodules improves readability and long-term maintainability for security-sensitive code.

## What Changes
- Convert `secrets` into a folder module under `crates/bastion-storage/src/secrets/`
- Split implementation into focused submodules (`crypto`, `keyring`, `keypack`, `io`)
- Keep the existing public surface stable (`bastion_storage::secrets::*`) and preserve behavior (no crypto/semantic changes)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-storage/src/secrets.rs`, `crates/bastion-storage/src/secrets/*.rs`

## Compatibility / Non-Goals
- No changes intended to encryption/decryption behavior, AAD compatibility, key derivation, keypack format, or on-disk file formats beyond structural movement.

