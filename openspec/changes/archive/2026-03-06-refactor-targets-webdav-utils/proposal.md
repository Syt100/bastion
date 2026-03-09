# Change: Refactor WebDAV helper utilities in bastion-targets

## Why
`crates/bastion-targets/src/webdav.rs` and `crates/bastion-targets/src/webdav_client.rs` currently define identical `redact_url` helpers. Centralizing shared helpers reduces duplication and prevents subtle drift in logging and diagnostics.

## What Changes
- Deduplicate the WebDAV URL redaction helper so it is implemented once and reused
- Preserve behavior and existing public API (`WebdavClient`, `WebdavCredentials`, `webdav::store_run`)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-targets/src/webdav.rs`, `crates/bastion-targets/src/webdav_client.rs`

## Compatibility / Non-Goals
- No behavior changes intended for WebDAV uploads/downloads, retry policy, or URL formatting semantics.

