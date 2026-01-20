---
## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and spec deltas (`backup-format`, `sources`, `backend`, `web-ui`)
- [x] 1.2 Run `openspec validate add-backup-format-raw-tree --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Core - Manifest and entries index extensions
- [x] 2.1 Add manifest field(s) to record artifact format (`archive_v1` vs `raw_tree_v1`)
- [x] 2.2 Extend entries index records to include metadata fields (best-effort; backward compatible)
- [x] 2.3 Commit core format changes (detailed message)

## 3. Backup - Build raw_tree_v1 for filesystem jobs
- [ ] 3.1 Implement raw-tree builder that writes `data/<path>` and metadata records
- [ ] 3.2 Ensure symlink/hardlink policies are respected and recorded
- [ ] 3.3 Store raw-tree runs to LocalDir and WebDAV targets (recursive copy/upload)
- [ ] 3.4 Commit backup changes (detailed message)

## 4. Restore - Support raw_tree_v1
- [ ] 4.1 Implement raw-tree restore that streams file bytes from `data/<path>` and writes via restore sinks
- [ ] 4.2 Apply metadata best-effort for filesystem sinks; write `.bastion-meta` for WebDAV sinks
- [ ] 4.3 Commit restore changes (detailed message)

## 5. Web UI - Format selection
- [ ] 5.1 Add job editor option to choose artifact format (default archive)
- [ ] 5.2 Disable encryption UI when raw-tree is selected; adjust validation/i18n
- [ ] 5.3 Commit UI changes (detailed message)

## 6. Verification
- [ ] 6.1 Run `cargo test --workspace`
- [ ] 6.2 Run `npm test --prefix ui`
- [ ] 6.3 Run `npm run type-check --prefix ui`
