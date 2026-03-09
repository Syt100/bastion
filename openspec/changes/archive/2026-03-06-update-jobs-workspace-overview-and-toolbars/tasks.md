## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (overview summary + compact action headers + mobile)
- [x] 1.3 Add `web-ui` spec delta (overview run summary + history/data compact toolbars)
- [x] 1.4 Run `openspec validate update-jobs-workspace-overview-and-toolbars --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [x] 2.1 Add Overview “Run Summary (last 7 days)” block (latest run + compact 7d metrics)
- [x] 2.2 Remove the summary card grid from History; keep runs list
- [x] 2.3 Move History actions (Refresh) into the runs list panel header (no standalone action row)
- [x] 2.4 Refactor Data page so Retention/Snapshots actions live in their panel headers
- [x] 2.5 Ensure mobile layout does not wrap toolbars into multi-line headers (use icon/overflow)
- [x] 2.6 Add/update i18n strings for new labels (7d summary, latest run)

## 3. Tests / Validation
- [x] 3.1 Update/add unit tests for Overview summary rendering (no runs vs has runs)
- [x] 3.2 Update/add unit tests ensuring History/Data do not render standalone action rows
- [x] 3.3 Run `npm test --prefix ui`
- [x] 3.4 Run `bash scripts/ci.sh`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
