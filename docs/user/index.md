# User manual

Bastion is a self-hosted backup orchestrator for small deployments (personal / family / small teams).

If you are new to Bastion, start with: [Concepts and terminology](/user/concepts).

## Components

- **Hub**: the main service (HTTP API + Web UI). Stores metadata in SQLite and manages encrypted secrets.
- **Agent** (optional): connects to the Hub and runs jobs on another machine.

## Typical workflow

1. [Start the Hub and finish first-run setup](/user/getting-started).
2. (Optional) [Enroll Agents](/user/agents) for multi-node backups.
3. Create [Jobs](/user/jobs) and run them.
4. Monitor [Runs](/user/runs) and use [Restore and verify](/user/restore-verify) for recovery and integrity checks.
5. Manage backup outputs in [Backup snapshots](/user/backup-snapshots) (pin / delete / retention).
6. Configure [Storage (WebDAV)](/user/storage) if you back up to a remote target.
7. (Optional) Configure [Notifications](/user/operations/notifications).

## Reference

- [Bulk operations](/user/bulk-operations) (labels, sync config, distribute WebDAV credentials, deploy jobs)
- [Operations](/user/operations/defaults) (defaults / upgrade & rollback / reverse proxy / runtime config / logging / observability)
- [Recipes](/user/recipes/vaultwarden)
