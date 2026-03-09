## 1. Spec
- [x] 1.1 Add backend + web-ui spec deltas for About page and build metadata
- [x] 1.2 Run `openspec validate add-about-page-build-info --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Backend: extend `/api/system` with build metadata fields (version + build time)
- [x] 2.2 Backend: generate build time safely for local/dev/source builds
- [x] 2.3 Web UI: add Settings -> About page (requires login)
- [x] 2.4 Web UI: embed UI version + build time into the bundle

## 3. Validation
- [x] 3.1 Ensure `bash scripts/ci.sh` passes

## 4. GitHub
- [x] 4.1 Ensure release workflow uses tag version for About page metadata

## 5. Commits
- [x] 5.1 Commit implementation changes (detailed message)
- [x] 5.2 Mark OpenSpec tasks complete and commit
