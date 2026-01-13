## 1. Spec
- [ ] 1.1 Add backend + web-ui spec deltas for About page and build metadata
- [ ] 1.2 Run `openspec validate add-about-page-build-info --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Backend: extend `/api/system` with build metadata fields (version + build time)
- [ ] 2.2 Backend: generate build time safely for local/dev/source builds
- [ ] 2.3 Web UI: add Settings -> About page (requires login)
- [ ] 2.4 Web UI: embed UI version + build time into the bundle

## 3. Validation
- [ ] 3.1 Ensure `bash scripts/ci.sh` passes

## 4. GitHub
- [ ] 4.1 Ensure release workflow uses tag version for About page metadata

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message)
- [ ] 5.2 Mark OpenSpec tasks complete and commit
