# Change: Refactor restore entries index module structure

## Why
`crates/bastion-backup/src/restore/entries_index.rs` currently contains a mix of concerns (types, remote index fetching, and local index listing). Splitting it into focused submodules improves maintainability and makes it easier to evolve each concern independently.

## What Changes
- Convert `restore/entries_index.rs` into a folder module (`restore/entries_index/mod.rs`)
- Split the implementation into focused submodules (types, fetch, list)
- Preserve existing public API and behavior (`restore::entries_index::*`)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/restore/entries_index.rs` and new `crates/bastion-backup/src/restore/entries_index/` submodules

## Compatibility / Non-Goals
- No behavior changes intended for entries index download, caching, filtering, sorting, or pagination.

