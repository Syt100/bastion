## 1. Spec Foundation

- [ ] 1.1 Finalize proposal, design, and spec deltas for `fleet-management`, `integrations-management`, `system-management`, and `control-plane`
- [ ] 1.2 Run `openspec validate redesign-fleet-integrations-and-system-management --strict`

## 2. Control-Plane And Read Models

- [ ] 2.1 Add public base URL runtime-config persistence plus effective-value exposure for authenticated UI clients
- [ ] 2.2 Implement aggregated Fleet and Integrations read models with backend tests for healthy, empty, and degraded states

## 3. Fleet UI

- [ ] 3.1 Build the Fleet landing page with health summary, onboarding rail, and fleet list
- [ ] 3.2 Add dedicated agent detail pages and route existing agent entry points into Fleet
- [ ] 3.3 Update generated onboarding commands and related copy to use the effective public base URL

## 4. Integrations And System UI

- [ ] 4.1 Build the Integrations index plus storage, notifications, and distribution management subsections
- [ ] 4.2 Build the System index plus runtime, maintenance, appearance, and about subsections
- [ ] 4.3 Migrate current Settings and Agents entry points to canonical routes, keeping only minimal temporary client-side aliases where cleanup cannot land in one step

## 5. Validation

- [ ] 5.1 Add or update UI tests for fleet onboarding, fleet detail, integrations grouping, and system route behavior
- [ ] 5.2 Run targeted backend and UI tests plus broader verification for navigation and generated-command correctness
