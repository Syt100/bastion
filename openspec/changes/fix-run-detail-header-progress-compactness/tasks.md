## 1. Spec
- [ ] 1.1 Add web-ui spec delta for header/status placement, stage help access, and compact layout tweaks
- [ ] 1.2 Run `openspec validate fix-run-detail-header-progress-compactness --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Run Detail header: move status badge to the right side
- [ ] 2.2 Overview target path: wrap (no ellipsis)
- [ ] 2.3 Progress panel: restore Scan/Packaging help access; render Upload 100% as finished; tighten spacing

## 3. Tests
- [ ] 3.1 Update/add unit tests for RunProgressPanel/RunDetail as needed

## 4. Validation
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `npm --prefix ui run build`
- [ ] 4.3 Run `cargo test -p bastion-http`

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)
