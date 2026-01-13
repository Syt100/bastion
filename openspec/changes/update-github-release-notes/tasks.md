## 1. Spec
- [ ] 1.1 Add `dev-workflow` spec delta for release notes generation
- [ ] 1.2 Run `openspec validate update-github-release-notes --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Update `.github/workflows/release.yml` to generate release notes from git history

## 3. Validation
- [ ] 3.1 Verify the notes script works locally for a tag (e.g. `v0.1.0`)

## 4. GitHub
- [ ] 4.1 Push changes to `origin/main`

## 5. Commits
- [ ] 5.1 Commit workflow changes (detailed message)
- [ ] 5.2 Mark OpenSpec tasks complete and commit
