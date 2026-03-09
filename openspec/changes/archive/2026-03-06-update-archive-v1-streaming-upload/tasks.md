## 1. Spec
- [x] 1.1 Add `backup-format` spec delta for rolling part storage for `archive_v1`
- [x] 1.2 Add `targets-webdav` spec delta for streaming part upload + local part cleanup
- [x] 1.3 Add `targets-local-dir` spec delta for streaming part copy/move + local part cleanup
- [x] 1.4 Add design.md decisions for queue/backpressure + completion ordering
- [x] 1.5 Run `openspec validate update-archive-v1-streaming-upload --strict`
- [x] 1.6 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add an archive builder hook to emit "part ready" events when a part is finalized
- [x] 2.2 Implement bounded rolling part storage for WebDAV (upload on finalize, then delete local part)
- [x] 2.3 Implement bounded rolling part storage for LocalDir (copy/move on finalize, then delete local part)
- [x] 2.4 Preserve ordering: upload/write `entries.jsonl.zst`, then `manifest.json`, then `complete.json`
- [x] 2.5 Ensure upload progress totals/done remain meaningful when rolling part storage is enabled

## 3. Tests
- [x] 3.1 Add tests covering rolling part deletion and resumability-by-size
- [x] 3.2 Add a regression test that packaging does not leave all parts on disk simultaneously (best-effort)

## 4. Validation
- [x] 4.1 Run `./scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit implementation changes (detailed message with Modules/Tests)
