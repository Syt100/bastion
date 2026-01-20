# Design: Streaming Restore Engine

## Overview
This change introduces a restore engine that is:
- **source-agnostic** (WebDAV vs local dir),
- **sink-agnostic** (local fs now; WebDAV/agent later),
- **streaming** (bounded memory; no full-payload staging requirement).

The engine operates on a single logical "restore stream" derived from run artifacts.

## Core Abstractions

### ArtifactSource (read side)
Responsibilities:
- Read `manifest.json` and `entries.jsonl.zst`
- Provide access to payload content as a stream (for archive parts)

Initial implementations:
- `LocalDirSource`: reads from a run directory on the local filesystem
- `WebdavSource`: reads from WebDAV using existing WebDAV client helpers

### RestoreSink (write side)
Responsibilities:
- Create directories and write regular files
- Apply conflict policy (`overwrite|skip|fail`) and basic metadata (mtime)

Initial implementations:
- `LocalFsSink`: writes to a destination directory on the local filesystem (current behavior)

### RestoreEngine
Responsibilities:
- Determine restore mode from manifest (still archive-only in this change)
- Apply selection filter (paths/dir subtrees)
- Stream entries from the archive and write to sink

## Archive Restore Path Handling
- Input archive paths are validated and normalized as *relative* paths before writing.
- The sink is given normalized relative paths; the sink is responsible for joining with its own base destination.

## Metadata
This refactor keeps the existing behavior. Metadata is applied on a best-effort basis:
- Preserve mtime where possible.
- Permissions/xattrs are explicitly out of scope until the raw-tree format change.

## Error Handling
- Errors are surfaced with enough context (entry path + operation stage) to be recorded as operation events.
- Partial writes are avoided by using atomic temp files where feasible (sink responsibility).

