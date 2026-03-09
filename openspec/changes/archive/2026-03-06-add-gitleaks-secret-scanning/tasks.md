## 1. Spec
- [x] 1.1 Add OpenSpec change proposal and tasks
- [x] 1.2 Run `openspec validate add-gitleaks-secret-scanning --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add gitleaks scan step to `scripts/ci.sh` (auto-install pinned gitleaks via Go when missing)
- [x] 2.2 Add gitleaks scan step to `scripts/ci.ps1` (auto-install pinned gitleaks via Go when missing)
- [x] 2.3 Confirm no custom gitleaks config is needed (no false positives)

## 3. Validation
- [x] 3.1 Run gitleaks scan locally and confirm it passes for this repo

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
- [x] 4.2 Mark OpenSpec tasks complete and commit
