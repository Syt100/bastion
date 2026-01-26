## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for VitePress + GitHub Pages docs site at `/<repo>/docs/`
- [x] 1.2 Run `openspec validate add-docs-site-vitepress --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add VitePress scaffolding under `docs/` (`docs/package.json`, `docs/.vitepress/config.ts`, `docs/index.md`)
- [x] 2.2 Add GitHub Pages workflow for docs build + deploy to `/docs/`
- [x] 2.3 Update gitignore(s) to exclude docs build artifacts and `docs/node_modules`
- [x] 2.4 Add a short doc entry in root `README.md` pointing to the hosted docs site

## 3. Validation
- [x] 3.1 Run `npm ci --prefix docs` and `npm run build --prefix docs`
- [x] 3.2 Ensure the built output works under the `/docs/` base path

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
- [x] 4.2 Mark OpenSpec tasks complete and commit
