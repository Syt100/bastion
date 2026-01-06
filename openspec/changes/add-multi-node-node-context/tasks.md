## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for node context (node switcher, `/n/:nodeId/**` routing, global vs node pages, per-node job UX)
- [x] 1.2 Run `openspec validate add-multi-node-node-context --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI
- [x] 2.1 Add node context selector (Hub + Agents) in the app chrome
- [x] 2.2 Add `/n/:nodeId/**` routes and ensure refresh/deep links preserve node context
- [x] 2.3 In node context, filter Jobs/Runs/Restore/Verify to the selected node and default new jobs to that node
- [x] 2.4 Keep global pages (Agents, global settings) accessible and clearly scoped
- [x] 2.5 Add/adjust unit tests for node routing and filtering

## 3. Validation
- [x] 3.1 Run `npm run lint --prefix ui`
- [x] 3.2 Run `npm test --prefix ui`
- [x] 3.3 Run `npm run build --prefix ui`

## 4. Commits
- [x] 4.1 Commit Web UI node-context changes (detailed message with Modules/Tests)
