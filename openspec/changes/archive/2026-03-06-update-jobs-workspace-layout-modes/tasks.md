## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (layout modes + list/table views + persistence + mobile)
- [x] 1.3 Add `web-ui` spec delta (layout modes, table view constraints, mobile behavior)
- [x] 1.4 Run `openspec validate update-jobs-workspace-layout-modes --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [x] 2.1 Add Jobs workspace layout mode state (split/list/detail) with desktop-only persistence
- [x] 2.2 Add UI controls to toggle layout modes (hide list / hide detail / back to split)
- [x] 2.3 Implement jobs list List/Table view toggle
- [x] 2.4 Implement table view (columns + sorting + per-row actions) and gate it to List-only mode
- [x] 2.5 Ensure mobile keeps single-column navigation and does not surface desktop-only toggles
- [x] 2.6 Add/refresh i18n strings for new labels/tooltips

## 3. Tests / Validation
- [x] 3.1 Add unit tests covering layout mode toggles and persistence behavior
- [x] 3.2 Add unit tests covering table view gating (desktop only, list-only only)
- [x] 3.3 Run `npm test --prefix ui`
- [x] 3.4 Run `bash scripts/ci.sh`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
