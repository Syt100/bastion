## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (shared scaffold + pagination + empty-state variants)
- [x] 1.3 Add `web-ui` spec delta for list scaffold, pagination consistency, and visual-noise reduction
- [x] 1.4 Run `openspec validate refactor-web-ui-list-page-scaffold-and-pagination --strict`

## 2. Implementation (Web UI)
- [x] 2.1 Introduce shared list-page scaffold primitives in `ui/src/components/list/`
- [x] 2.2 Add shared pagination wrapper/contract and migrate page-level usage
- [x] 2.3 Extend `AppEmptyState` with `card`/`inset`/`plain` variants
- [x] 2.4 Migrate Notifications Queue to shared scaffold + unified pagination behavior
- [x] 2.5 Migrate Agents list page to shared scaffold + unified pagination behavior
- [x] 2.6 Migrate Jobs workspace list panel to shared scaffold regions where applicable
- [x] 2.7 Remove duplicate page-level wrappers made obsolete by the scaffold migration

## 3. Tests / Validation
- [x] 3.1 Add/update unit tests for scaffold region rendering (selection/toolbar/content/footer)
- [x] 3.2 Add/update tests for pagination behavior parity across migrated pages
- [x] 3.3 Add/update tests for empty-state variant behavior in card vs embedded contexts
- [x] 3.4 Run `npm test --prefix ui`
- [x] 3.5 Run `bash scripts/ci.sh`
