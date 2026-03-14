## Context

The current product spreads control-plane operations across several mismatched surfaces:

- `Agents` is both a list and an onboarding page
- storage management is partly node-scoped and partly global
- notifications, runtime config, and maintenance live under a broad `Settings` area that mixes daily and infrequent work
- generated onboarding commands use browser origin assumptions instead of explicit product configuration

This change reorganizes those concerns into clearer operational domains:

- `Fleet` for nodes/agents and onboarding
- `Integrations` for storage, notifications, and distribution
- `System` for low-frequency product administration

## Goals / Non-Goals

**Goals:**
- make agent onboarding and fleet health coherent in one surface
- group integrations by operator task rather than by existing page placement
- make low-frequency system configuration clearly distinct from daily operational workflows
- provide an explicit public base URL for generated commands and links

**Non-Goals:**
- changing agent enrollment protocol semantics
- redesigning command-center or job/run workspaces in this change
- replacing existing storage/notification business rules

## Decisions

### 1. `Agents` will become `Fleet`

The surface will combine:

- health summary
- onboarding rail / token creation
- agent list
- agent detail

Rationale:
- nodes are an operational fleet, not just a table of clients
- onboarding must live beside health and sync state so the page scales from empty state to mature deployments

Alternatives considered:
- keep `Agents` as-is and only polish empty state copy
  - rejected because the current grouping still leaves onboarding and operations disconnected

#### Fleet route family

The canonical Fleet route family will be:

- `/fleet`
- `/fleet/:agentId`

`/fleet` combines onboarding, summary, list, and bulk actions. `/fleet/:agentId` is the dedicated detail page for one fleet member.

Temporary migration aliases:

- `/agents` -> `/fleet` only while old internal navigation/tests are still being updated
- agent-specific actions that previously opened inline dialogs may resolve to `/fleet/:agentId` once parity exists

### 2. Storage, notifications, and distribution will live under `Integrations`

The product will treat these as external system integrations rather than generic settings.

Rationale:
- operators reason about these areas through external dependencies and delivery paths, not as abstract preferences

Alternatives considered:
- keep everything under `Settings`
  - rejected because it preserves the current high-frequency/low-frequency mismatch

#### Integrations route family

The canonical Integrations route family will be:

- `/integrations`
- `/integrations/storage`
- `/integrations/notifications`
- `/integrations/distribution`

Scope-aware subsections use query scope rather than node-prefixed paths, for example:

- `/integrations/storage?scope=hub`
- `/integrations/storage?scope=agent:edge-a`

Temporary migration aliases:

- `/settings/notifications` -> `/integrations/notifications` only while old internal references remain
- `/settings/notifications/channels` -> `/integrations/notifications/channels` only while old internal references remain
- `/settings/notifications/destinations` -> `/integrations/notifications/destinations` only while old internal references remain
- `/settings/notifications/templates` -> `/integrations/notifications/templates` only while old internal references remain
- `/settings/notifications/queue` -> `/integrations/notifications/queue` only while old internal references remain
- `/n/:nodeId/settings/storage` -> `/integrations/storage?scope=<mapped>` only while old node-scoped storage entry points remain

### 3. `System` will become a low-frequency administrative area

`System` will host:

- runtime configuration
- maintenance
- appearance
- about

Rationale:
- these areas matter, but they are not the daily operational center of the product

#### System route family

The canonical System route family will be:

- `/system`
- `/system/runtime`
- `/system/maintenance`
- `/system/appearance`
- `/system/about`

Temporary migration aliases:

- `/settings` -> `/system`
- `/settings/hub-runtime-config` -> `/system/runtime`
- `/settings/maintenance/cleanup` -> `/system/maintenance/cleanup`
- `/settings/appearance` -> `/system/appearance`
- `/settings/about` -> `/system/about`

### 4. Generated operator-facing URLs will use an explicit public base URL

The system will support a configurable public base URL with clear precedence and expose the effective value to authenticated UI clients that need to generate commands.

Rationale:
- browser origin is not reliable in reverse-proxy, SSH-forwarded, or local-dev setups

Alternatives considered:
- continue using `window.location.origin`
  - rejected because it produces incorrect operator instructions in common deployment/debugging patterns

#### Public base URL contract

`public_base_url` becomes an optional runtime-config field with env/CLI precedence aligned to existing runtime config behavior.

Rules:

- env/CLI field name: `BASTION_PUBLIC_BASE_URL`
- accepted values may include a path prefix, for example `https://backup.example.com/bastion`
- values are normalized without a trailing slash before persistence/exposure
- precedence is `CLI/ENV override > saved DB value > unset`
- when unset, the system exposes explicit absence instead of silently manufacturing an operator-facing command URL from browser origin

To support non-System surfaces such as Fleet, the control plane exposes lightweight authenticated metadata dedicated to operator-facing link generation. This avoids forcing every page to fetch the full runtime-config document merely to render onboarding commands.

### 5. Fleet and Integrations pages will consume aggregated read models

The UI will consume dedicated view models that include:

- agent/fleet health summaries
- config drift / pending operations
- integration usage and validation signals
- distribution coverage or failure summaries

Rationale:
- these surfaces need cross-reference context and should not be assembled from many unrelated endpoints in the browser

#### Fleet read-model contract

Illustrative `/api/fleet` response:

```json
{
  "summary": {
    "total": 4,
    "online": 3,
    "offline": 1,
    "revoked": 0,
    "drifted": 1
  },
  "onboarding": {
    "public_base_url": "https://backup.example.com",
    "command_generation_ready": true
  },
  "items": [
    {
      "id": "edge-a",
      "name": "DB Node A",
      "status": "offline",
      "last_seen_at": 1759990000,
      "config_sync": {
        "state": "drifted",
        "last_error_kind": "send_failed"
      },
      "assigned_jobs_total": 6,
      "pending_tasks_total": 1
    }
  ]
}
```

Illustrative `/api/fleet/:agentId` response:

```json
{
  "agent": {
    "id": "edge-a",
    "name": "DB Node A",
    "status": "offline"
  },
  "sync": {
    "desired_snapshot_id": "cfg_1",
    "applied_snapshot_id": "cfg_0",
    "state": "drifted",
    "last_error_kind": "send_failed"
  },
  "recent_activity": [],
  "related_jobs": [],
  "capabilities": {
    "can_rotate_key": true,
    "can_revoke": true,
    "can_sync_now": true
  }
}
```

#### Integrations read-model contract

Illustrative `/api/integrations` response:

```json
{
  "storage": {
    "state": "ready",
    "summary": {
      "items_total": 3,
      "in_use_total": 2,
      "invalid_total": 1
    }
  },
  "notifications": {
    "state": "ready",
    "summary": {
      "destinations_total": 4,
      "recent_failures_total": 2,
      "queue_backlog_total": 5
    }
  },
  "distribution": {
    "state": "degraded",
    "summary": {
      "coverage_total": 8,
      "drifted_total": 1,
      "failed_total": 1
    }
  }
}
```

Each domain may be independently `ready`, `empty`, or `degraded` so a partial failure in notifications does not blank the storage or distribution sections.

## Risks / Trade-offs

- [Large IA shift may confuse existing users temporarily] → keep temporary aliases small, update labels carefully, and keep onboarding/help copy explicit during migration
- [Integrations aggregation may require new backend joins or summary materialization] → define narrow first contracts and optimize iteratively
- [Public base URL introduces precedence complexity] → align it with existing runtime-config precedence rules and surface the effective value explicitly in UI/API
- [Fleet detail pages may expose more data than current list-only workflows] → stage detail rollout behind the list/onboarding rebuild if necessary

## Migration Plan

1. Add Fleet, Integrations, and System routes and navigation metadata alongside existing pages.
2. Add public base URL persistence/effective-value exposure to runtime configuration.
3. Introduce Fleet list/detail and onboarding rail, then route current agent links into Fleet.
4. Introduce Integrations index and migrate storage/notifications/distribution pages underneath it.
5. Remove the old Settings area after internal entry points have been updated, leaving at most short-lived aliases during the transition.

Rollback:
- keep current `Agents` and `Settings` entry points until Fleet/Integrations/System routes are validated
- preserve browser-origin command generation as a temporary fallback only while the public-base-url contract rolls out

#### Rollout adjustment

The rollout will deliberately remove automatic browser-origin command generation as soon as the authenticated public-metadata contract exists. During any short compatibility window before the runtime-config field is fully wired, the Fleet UI may show an unresolved onboarding command state with configuration guidance, but it must not silently present `window.location.origin` as if it were canonical.

## Open Questions

- Should node-scoped storage management remain directly accessible as a Fleet child route, or only through Integrations with agent filters/context?
- Which integration health signals can be computed synchronously at page load versus requiring background refresh jobs later?
