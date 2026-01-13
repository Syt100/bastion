## 1. Spec
- [ ] 1.1 Add OpenSpec change proposal and tasks
- [ ] 1.2 Run `openspec validate add-gitleaks-secret-scanning --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Add gitleaks scan step to `scripts/ci.sh` (auto-install pinned gitleaks via Go when missing)
- [ ] 2.2 Add gitleaks scan step to `scripts/ci.ps1` (auto-install pinned gitleaks via Go when missing)
- [ ] 2.3 (Optional) Add a minimal repo config/ignore for false positives if needed

## 3. Validation
- [ ] 3.1 Run gitleaks scan locally and confirm it passes for this repo

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit

