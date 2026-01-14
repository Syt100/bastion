## 1. Spec
- [x] 1.1 Add Web UI spec deltas for navigation list dedup + drift guards
- [x] 1.2 Run `openspec validate refactor-web-ui-nav-dedup --strict`

## 2. Web UI (Notifications)
- [x] 2.1 Add shared Notifications navigation config
- [x] 2.2 Refactor Notifications index list + desktop tabs to use config
- [x] 2.3 Add/adjust unit tests to guard router/nav drift

## 3. Web UI (Locales)
- [x] 3.1 Add shared locale options helper derived from `supportedLocales`
- [x] 3.2 Refactor AppShell/AuthLayout/App.vue to use helper/mappings
- [x] 3.3 Add unit tests to guard locale option/mapping drift

## 4. Validation
- [ ] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit the spec proposal (detailed message)
- [x] 5.2 Commit Notifications navigation refactor (detailed message with tests)
- [x] 5.3 Commit locale options refactor (detailed message with tests)
- [ ] 5.4 Mark tasks complete and commit
