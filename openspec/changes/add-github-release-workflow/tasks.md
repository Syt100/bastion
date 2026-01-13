## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for GitHub Releases
- [x] 1.2 Run `openspec validate add-github-release-workflow --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add GitHub Actions `release` workflow (Linux x64 + Windows x64)

## 3. Validation
- [x] 3.1 Verify local release build path: `npm ci --prefix ui && npm run build-only --prefix ui && cargo build -p bastion --release --features embed-ui`

## 4. GitHub
- [x] 4.1 Push changes to `origin/main`

## 5. Commits
- [x] 5.1 Commit workflow changes (detailed message)
- [x] 5.2 Mark OpenSpec tasks complete and commit
