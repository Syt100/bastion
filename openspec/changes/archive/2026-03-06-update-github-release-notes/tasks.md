## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for release notes generation
- [x] 1.2 Run `openspec validate update-github-release-notes --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Update `.github/workflows/release.yml` to generate release notes from git history

## 3. Validation
- [x] 3.1 Verify the notes script works locally for a tag (e.g. `v0.1.0`)

## 4. GitHub
- [x] 4.1 Push changes to `origin/main`

## 5. Commits
- [x] 5.1 Commit workflow changes (detailed message)
- [x] 5.2 Mark OpenSpec tasks complete and commit
