## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (layout modes + list/table views + persistence + mobile)
- [x] 1.3 Add `web-ui` spec delta (layout modes, table view constraints, mobile behavior)
- [x] 1.4 Run `openspec validate update-jobs-workspace-layout-modes --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [ ] 2.1 Add Jobs workspace layout mode state (split/list/detail) with desktop-only persistence
- [ ] 2.2 Add UI controls to toggle layout modes (hide list / hide detail / back to split)
- [ ] 2.3 Implement jobs list List/Table view toggle
- [ ] 2.4 Implement table view (columns + sorting + per-row actions) and gate it to List-only mode
- [ ] 2.5 Ensure mobile keeps single-column navigation and does not surface desktop-only toggles
- [ ] 2.6 Add/refresh i18n strings for new labels/tooltips

## 3. Tests / Validation
- [ ] 3.1 Add unit tests covering layout mode toggles and persistence behavior
- [ ] 3.2 Add unit tests covering table view gating (desktop only, list-only only)
- [ ] 3.3 Run `npm test --prefix ui`
- [ ] 3.4 Run `bash scripts/ci.sh`

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
