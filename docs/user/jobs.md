# Jobs

This document covers jobs, schedules/timezones, and deploying jobs to multiple nodes.

## Where jobs run

A job can be scoped to:

- **Hub (local)**: job runs on the Hub node
- **Agent node**: job runs on a specific enrolled agent

In the Web UI, the Jobs list can be viewed globally or in a node context (when navigating via a node).

## Scheduling

Each job has:

- **Schedule mode**: manual / simple / cron
- **Schedule timezone**: an explicit IANA timezone string used to interpret the schedule (independent of Hub/Agent system timezones)
- **Overlap policy**: how to handle triggers while a run is already executing (queue vs reject)

## Deploy (clone) a job to nodes

The Web UI provides a bulk “deploy to nodes” action to clone an existing job onto many agents.

UI entry point:

- **Jobs** → pick a job → **Deploy to nodes**

What deploy does:

- Creates a new job for each selected agent.
- Preserves the source job’s spec, schedule, schedule timezone, and overlap policy.
- Performs per-node validation (for example, required node-scoped secrets must exist).
- Triggers/requests a config sync after creating each job (offline agents will apply on next connect).

Naming template:

- Default template: `{name} ({node})`
- Supported placeholders: `{name}`, `{node}`
- If the generated name collides on a node, the system auto-suffixes (`#2`, `#3`, …).

Preview:

- The deploy modal supports a preview step that shows planned names and per-node validation results.
- After starting the deploy, progress and per-node outcomes are tracked in **Settings → Bulk operations**.

## Backup snapshots and retention

Successful runs can produce a **backup snapshot** (the stored backup output).

- Manage snapshots (pin/delete/delete log): see [Backup snapshots](/user/backup-snapshots).
- Configure retention (keep last / keep days): in the job editor, under **Retention**.
