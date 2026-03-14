## Why

Run inspection is currently buried under job-centric flows and often rendered in modal-heavy layouts. That makes failure diagnosis slower than it should be, especially on mobile, even though "why did this run fail?" is one of the most important operator questions in the product.

## What Changes

- Promote `Runs` to a first-class top-level workspace with its own index and deep-linkable detail pages.
- Replace modal-first run inspection with a dedicated run workspace on desktop and mobile.
- Add a structured run diagnostics model that surfaces root cause, failure stage, hint, and first-error location before verbose raw payloads.
- Add a server-driven event console contract for filtering, pagination, first-error navigation, and deep linking.
- Make restore, verify, cancel, and related actions available from the dedicated run workspace instead of hiding them behind job-local inspection flows.

## Capabilities

### New Capabilities
- `runs-workspace`: global Runs index, dedicated run detail route model, desktop/mobile inspection flows, and operator actions from run context
- `run-diagnostics`: structured failure summary, event-console contract, diagnostic prioritization, and progressive disclosure of raw payloads

### Modified Capabilities

## Impact

- Affected code:
  - `ui/src/components/runs`, `ui/src/views`, router, stores, and i18n
  - run/event/operation APIs in `crates/bastion-http`
  - supporting query/aggregation logic for run view models and event filtering
- Affected APIs:
  - top-level Runs list
  - dedicated run detail view model
  - server-filterable run events endpoint
- Product impact:
  - run diagnosis becomes a first-class workflow instead of a subordinate modal path
