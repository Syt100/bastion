## 1. Spec
- [ ] 1.1 Add `sources` delta (multi-path + archive-path include/exclude)
- [ ] 1.2 Add `backup-format` delta (entries index path semantics)
- [ ] 1.3 Add `control-plane` delta (fs browse + entries browse + partial restore APIs)
- [ ] 1.4 Add `hub-agent` delta (agent fs browse forwarding)
- [ ] 1.5 Add `web-ui` delta (multi-source picker + partial restore wizard)
- [ ] 1.6 Run `openspec validate update-filesystem-multi-path-and-partial-restore --strict`
- [ ] 1.7 Commit the spec proposal (detailed message)

## 2. Core: Filesystem Multi-Path Backup (Rust)
- [ ] 2.1 Extend job spec: `filesystem.source.paths` (+ legacy `root` alias)
- [ ] 2.2 Implement multi-path traversal (file + dir) with stable archive-path mapping
- [ ] 2.3 Apply include/exclude against archive paths; keep policies (symlink/hardlink/error)
- [ ] 2.4 Deduplicate overlapping sources with sample-limited warnings
- [ ] 2.5 Update/extend unit tests in `bastion-backup`
- [ ] 2.6 Commit the core changes (detailed message)

## 3. Backend: Browse APIs + Partial Restore (Rust)
- [ ] 3.1 Add node-scoped filesystem list API (hub implementation)
- [ ] 3.2 Extend Hub↔Agent protocol to support agent filesystem listing (request/response + timeout)
- [ ] 3.3 Add run entries browsing API (prefix + pagination + search/filters)
- [ ] 3.4 Add restore request support for path selection; implement partial restore filtering
- [ ] 3.5 Add/extend unit tests for APIs + restore filtering
- [ ] 3.6 Commit backend changes (detailed message)

## 4. Web UI: Multi-Source Picker + Partial Restore
- [ ] 4.1 Add reusable filesystem/archive browser modals (responsive, search/filters, hide dotfiles)
- [ ] 4.2 Update Job Editor filesystem source to support multiple paths + browse
- [ ] 4.2a Improve Job Editor validation UX (required marks, inline errors, help text layout)
- [ ] 4.3 Update Restore wizard to browse archived paths and select subset
- [ ] 4.4 Add “single directory” mode and use it for LocalDir `base_dir` browse
- [ ] 4.5 Update/extend unit tests
- [ ] 4.6 Commit UI changes (detailed message)

## 5. Validation
- [ ] 5.1 Run `cargo fmt --all`
- [ ] 5.2 Run `cargo clippy --all-targets --all-features`
- [ ] 5.3 Run `cargo test --all`
- [ ] 5.4 Run `npm run lint --prefix ui`
- [ ] 5.5 Run `npm run test --prefix ui`
- [ ] 5.6 Run `npm run build --prefix ui`
