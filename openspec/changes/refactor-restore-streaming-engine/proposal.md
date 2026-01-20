# Change: Refactor restore into a streaming engine with pluggable sources and sinks

## Why
Restore is currently implemented as:
- resolve the run target (WebDAV or LocalDir),
- fetch all payload parts to local staging (when WebDAV),
- unpack a tar stream into a local destination directory (`unpack_in`).

This creates unnecessary disk pressure on the Hub and makes it difficult to support new restore destinations (e.g. WebDAV prefix, Agent local filesystem) because the unpacker is tightly coupled to the local filesystem.

## What Changes
- Introduce internal abstractions for restore:
  - **ArtifactSource**: reads run artifacts (manifest, entries index, payload parts) as streams.
  - **RestoreSink**: writes restored paths (dirs/files/links + metadata) to a destination backend.
  - **RestoreEngine**: consumes an `ArtifactSource` and writes to a `RestoreSink` in a streaming manner.
- Keep behavior the same for the existing restore API (restore to a local filesystem directory), but implement it via the new `RestoreEngine` and a `LocalFsSink`.
- Preserve conflict policy and selection semantics.

## Impact
- Affected specs: `backend`
- Affected code:
  - Restore: `crates/bastion-backup/src/restore/*`
  - Targets access: `crates/bastion-backup/src/restore/access.rs`, `crates/bastion-backup/src/restore/parts.rs`

## Compatibility / Non-Goals
- No new restore destinations in this change (WebDAV destination + Agent destination are handled in separate changes).
- No changes to run artifact format or manifest schema.
- No Hub↔Agent protocol changes.

