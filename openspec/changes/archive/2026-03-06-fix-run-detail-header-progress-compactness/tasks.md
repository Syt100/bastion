## 1. Spec
- [x] 1.1 Add web-ui spec delta for header/status placement, stage help access, and compact layout tweaks
- [x] 1.2 Run `openspec validate fix-run-detail-header-progress-compactness --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Run Detail header: move status badge to the right side
- [x] 2.2 Overview target path: wrap (no ellipsis)
- [x] 2.3 Progress panel: restore Scan/Packaging help access; render Upload 100% as finished; tighten spacing

## 3. Tests
- [x] 3.1 Update/add unit tests for RunProgressPanel/RunDetail as needed

## 4. Validation
- [x] 4.1 Run `npm test --prefix ui`
- [x] 4.2 Run `npm --prefix ui run build`
- [x] 4.3 Run `cargo test -p bastion-http`

## 5. Commits
- [x] 5.1 Commit implementation changes (detailed message with Modules/Tests)
