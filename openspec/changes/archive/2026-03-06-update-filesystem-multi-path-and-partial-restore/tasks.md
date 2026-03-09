## 1. Spec
- [x] 1.1 Add `sources` delta (multi-path + archive-path include/exclude)
- [x] 1.2 Add `backup-format` delta (entries index path semantics)
- [x] 1.3 Add `control-plane` delta (fs browse + entries browse + partial restore APIs)
- [x] 1.4 Add `hub-agent` delta (agent fs browse forwarding)
- [x] 1.5 Add `web-ui` delta (multi-source picker + partial restore wizard)
- [x] 1.6 Run `openspec validate update-filesystem-multi-path-and-partial-restore --strict`
- [x] 1.7 Commit the spec proposal (detailed message)

## 2. Core: Filesystem Multi-Path Backup (Rust)
- [x] 2.1 Extend job spec: `filesystem.source.paths` (+ legacy `root` alias)
- [x] 2.2 Implement multi-path traversal (file + dir) with stable archive-path mapping
- [x] 2.3 Apply include/exclude against archive paths; keep policies (symlink/hardlink/error)
- [x] 2.4 Deduplicate overlapping sources with sample-limited warnings
- [x] 2.5 Update/extend unit tests in `bastion-backup`
- [x] 2.6 Commit the core changes (detailed message)

## 3. Backend: Browse APIs + Partial Restore (Rust)
- [x] 3.1 Add node-scoped filesystem list API (hub implementation)
- [x] 3.2 Extend Hub↔Agent protocol to support agent filesystem listing (request/response + timeout)
- [x] 3.3 Add run entries browsing API (prefix + pagination + search/filters)
- [x] 3.4 Add restore request support for path selection; implement partial restore filtering
- [x] 3.5 Add/extend unit tests for APIs + restore filtering
- [x] 3.6 Commit backend changes (detailed message)

## 4. Web UI: Multi-Source Picker + Partial Restore
- [x] 4.1 Add reusable filesystem/archive browser modals (responsive, search/filters, hide dotfiles)
- [x] 4.2 Update Job Editor filesystem source to support multiple paths + browse
- [x] 4.2a Improve Job Editor validation UX (required marks, inline errors, help text layout)
- [x] 4.2b Improve Job Editor review step (human-readable summary)
- [x] 4.2c Polish Job Editor step layouts (modern, concise)
- [x] 4.3 Update Restore wizard to browse archived paths and select subset
- [x] 4.4 Add “single directory” mode and use it for LocalDir `base_dir` browse
- [x] 4.5 Update/extend unit tests
- [x] 4.6 Commit UI changes (detailed message)

## 5. Validation
- [x] 5.1 Run `cargo fmt --all`
- [x] 5.2 Run `cargo clippy --all-targets --all-features`
- [x] 5.3 Run `cargo test --all`
- [x] 5.4 Run `npm run lint --prefix ui`
- [x] 5.5 Run `npm run test --prefix ui`
- [x] 5.6 Run `npm run build --prefix ui`
