## 1. Spec
- [ ] 1.1 Add `dev-workflow` spec delta for publishing installers and macOS release assets
- [ ] 1.2 Run `openspec validate add-installers-and-macos-release --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Linux: add `.deb` packaging via `cargo-deb` (binary-only install)
- [ ] 2.2 Linux: add `.rpm` packaging via `cargo-generate-rpm` (binary-only install)
- [ ] 2.3 Windows: add `.msi` packaging via WiX (no PATH by default)
- [ ] 2.4 macOS: add x64 + arm64 release archives
- [ ] 2.5 Release: publish `sha256sums.txt` covering all assets
- [ ] 2.6 Docs: add install instructions for `.deb`/`.rpm`/`.msi`/macOS archives

## 3. Validation
- [ ] 3.1 Run `bash scripts/ci.sh`
- [ ] 3.2 Validate release packaging on each platform via GitHub Actions (tag or workflow_dispatch dry-run)

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit
