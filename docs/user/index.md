# User manual

Bastion is a self-hosted backup orchestrator for small deployments (personal / family / small teams).

## Components

- **Hub**: the main service (HTTP API + Web UI). Stores metadata in SQLite and manages encrypted secrets.
- **Agent** (optional): connects to the Hub and runs jobs on another machine.

## Typical workflow

1. [Start the Hub and finish first-run setup](/user/getting-started).
2. (Optional) [Enroll Agents](/user/agents) for multi-node backups.
3. Create [Jobs](/user/jobs) and run them.
4. Manage backup outputs in [Backup snapshots](/user/backup-snapshots) (pin / delete / retention).
5. Configure [Storage (WebDAV)](/user/storage) if you back up to a remote target.

## Reference

- [Bulk operations](/user/bulk-operations) (labels, “sync config now”, distribute WebDAV credentials, deploy jobs)
- [Operations](/user/operations/reverse-proxy) (reverse proxy / logging / data directory)
- [Recipes](/user/recipes/vaultwarden)

