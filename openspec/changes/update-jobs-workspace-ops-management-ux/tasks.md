## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (filters + chips + bulk actions + table affordances + split resizer + mobile actions)
- [x] 1.3 Add `web-ui` spec delta (ops management UX improvements)
- [x] 1.4 Run `openspec validate update-jobs-workspace-ops-management-ux --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [ ] 2.1 Add persisted split list width preference to `ui` store (desktop-only)
- [ ] 2.2 Implement split pane drag resize + width clamping + persistence
- [ ] 2.3 Add filters summary: results counter + active filter chips + clear-all affordance
- [ ] 2.4 Implement bulk selection:
  - [ ] Table view selection column + `SelectionToolbar`
  - [ ] List view "Select mode" toggle + row checkboxes (list-only layout)
- [ ] 2.5 Implement bulk actions: run now, archive, unarchive (confirmations; skip archived where required)
- [ ] 2.6 Table view UX: header click sorting + fixed name/actions columns + row open affordance
- [ ] 2.7 List view UX: hover quick actions (run now/edit/more) and list-only detail open affordance
- [ ] 2.8 Refresh clarity: tooltips/labels and accessible names for list vs detail refresh
- [ ] 2.9 Mobile: add `MobileTopBar` actions slot + optional sticky mode; move job actions into top bar on mobile
- [ ] 2.10 Add/refresh i18n strings for new labels/tooltips

## 3. Tests / Validation
- [ ] 3.1 Unit tests for new `ui` store persisted preference(s)
- [ ] 3.2 Unit tests for jobs workspace selection + bulk actions guardrails
- [ ] 3.3 Unit tests for filter chips/counters rendering and clearing behavior
- [ ] 3.4 Run `npm test --prefix ui`
- [ ] 3.5 Run `bash scripts/ci.sh`

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
