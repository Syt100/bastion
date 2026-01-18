## 1. Spec
- [x] 1.1 Add `hub-agent-protocol` + `web-ui` spec deltas for filesystem sorting
- [x] 1.2 Run `openspec validate add-fs-list-sorting --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - FS List Sorting
- [ ] 2.1 Add HTTP query params for sorting (`sort_by`, `sort_dir`)
- [ ] 2.2 Extend Hub↔Agent `FsList` request/response for sorting (additive optional fields)
- [ ] 2.3 Implement sorting in Agent and Hub local listing with stable cursor pagination
- [ ] 2.4 Add/adjust backend tests (cursor stability for each sort mode)

## 3. Web UI - FS Picker Sorting Controls
- [ ] 3.1 Add sort controls (field + direction) and show current sort state
- [ ] 3.2 Ensure sorting resets paging correctly (refresh from first page)
- [ ] 3.3 Add/adjust unit tests

## 4. Validation
- [ ] 4.1 Run `cargo test --workspace`
- [ ] 4.2 Run `npm test --prefix ui`
- [ ] 4.3 Run `npm run type-check --prefix ui`

## 5. Commits
- [ ] 5.1 Commit backend changes (detailed message)
- [ ] 5.2 Commit UI changes (detailed message)
