## 1. Spec
- [x] 1.1 Add spec deltas for agent labels + filtering
- [x] 1.2 Run `openspec validate add-agent-labels --type change --strict`

## 2. Backend
- [ ] 2.1 Storage: add `agent_labels` table + repo helpers
- [ ] 2.2 HTTP: include labels in agents list/get responses
- [ ] 2.3 HTTP: add label CRUD endpoints and label index endpoint (labels + counts)
- [ ] 2.4 HTTP: add label filtering (`labels[]`, `labels_mode=and|or`)
- [ ] 2.5 Add backend unit tests for validation + filtering semantics

## 3. Web UI
- [ ] 3.1 Extend agents store/types to include labels and filter params
- [ ] 3.2 Agents page: show labels and add label filter (AND/OR)
- [ ] 3.3 Agents page: add per-agent label editor
- [ ] 3.4 Add/adjust unit tests

## 4. Validation
- [ ] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [ ] 5.1 Commit spec proposal (detailed message)
- [ ] 5.2 Commit implementation (detailed message)
- [ ] 5.3 Mark tasks complete and commit
