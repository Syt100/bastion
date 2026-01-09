# Change: Refactor WebDAV client request helpers

## Why
`crates/bastion-targets/src/webdav_client.rs` repeats request setup logic (basic auth wiring, request builder patterns) across multiple methods. Extracting small helpers reduces duplication and keeps WebDAV client methods easier to read and maintain.

## What Changes
- Add small internal helpers for building authenticated WebDAV requests
- Preserve behavior and existing public API (`WebdavClient`, `WebdavCredentials`)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-targets/src/webdav_client.rs`

## Compatibility / Non-Goals
- No behavior changes intended for WebDAV HTTP methods, retry policy, timeouts, or logging.

