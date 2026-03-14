## Why

Agent onboarding, storage credentials, notification configuration, bulk distribution, and runtime settings are currently fragmented across `Agents`, node-scoped storage pages, and broad `Settings` navigation. That makes Bastion's control-plane operations feel scattered, and it creates real operator mistakes such as generating onboarding commands from the wrong browser origin.

## What Changes

- Recast `Agents` as a `Fleet` surface with health summary, onboarding rail, and dedicated agent detail pages.
- Group storage, notifications, and distribution workflows under an `Integrations` surface rather than scattering them across unrelated settings pages.
- Reduce `System` to low-frequency runtime, maintenance, appearance, and about surfaces instead of using `Settings` as a catch-all for daily operations.
- Introduce a configurable public base URL contract so generated onboarding and operator-facing commands use an intentional control-plane address.
- Add aggregated Fleet and Integrations view models so the UI can display usage, health, drift, and follow-up actions without stitching together many unrelated calls.

## Capabilities

### New Capabilities
- `fleet-management`: fleet health summary, onboarding flows, agent list/detail behavior, and fleet-oriented aggregated data contract
- `integrations-management`: integrations index, storage/notification/distribution management surfaces, and cross-reference health/usage summaries
- `system-management`: low-frequency system pages for runtime, maintenance, appearance, and about with clearer operator expectations

### Modified Capabilities
- `control-plane`: add the public base URL configuration and public metadata behavior needed by onboarding and operator-facing command generation

## Impact

- Affected code:
  - `ui/src/views/AgentsView.vue`, settings surfaces, navigation metadata, router, stores, and i18n
  - runtime config APIs and aggregated fleet/integration read models in `crates/bastion-http`
  - supporting persistence/query code as needed for effective public URL and integrations summaries
- Affected APIs:
  - fleet list/detail views
  - integrations summaries
  - runtime/public metadata for operator-facing URLs
- Product impact:
  - onboarding and system configuration become more intentional and less error-prone
