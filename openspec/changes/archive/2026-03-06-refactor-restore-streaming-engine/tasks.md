---
## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and spec deltas (`backend`)
- [x] 1.2 Run `openspec validate refactor-restore-streaming-engine --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - Restore engine refactor (no API changes)
- [x] 2.1 Introduce `ArtifactSource` abstraction for reading manifest/entries/payload streams
- [x] 2.2 Introduce `RestoreSink` abstraction for writing restored paths and applying conflict policy
- [x] 2.3 Implement `LocalDirSource` and `WebdavSource` (feature parity with current restore inputs)
- [x] 2.4 Implement `LocalFsSink` (restore to local directory)
- [x] 2.5 Refactor restore operation to use the streaming engine + sink (preserve selection/conflict semantics)
- [x] 2.6 Add targeted tests for streaming restore behavior (selection + conflict policy)
- [x] 2.7 Commit backend changes (detailed message)

## 3. Verification
- [x] 3.1 Run `cargo test --workspace`
- [x] 3.2 (Optional) Run `cargo clippy --workspace --all-targets`
