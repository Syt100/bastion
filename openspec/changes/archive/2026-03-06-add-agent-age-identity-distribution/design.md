# Design: Age Identity Distribution to Agents

## Overview
Encrypted restore on an Agent requires the Agent to have access to the age identity private key referenced by `manifest.pipeline.encryption_key`.

We treat the age identity as a secret that can be copied from Hub scope to Agent scope:
- Secret kind: `backup_age_identity`
- Secret name: `<key_name>`

## Distribution Model
- Distribution is **on-demand**:
  - when a restore is requested to execute on Agent `<agent_id>`,
  - and the run is encrypted with key `<key_name>`,
  - and the Agent does not already have `backup_age_identity/<key_name>`,
  - the Hub copies the secret from Hub scope to Agent scope and triggers a secrets snapshot refresh.

## Agent Storage
- Agents already persist secret snapshots encrypted-at-rest via `SecretsCrypto`.
- The age identity is persisted using the same mechanism, never logged or emitted back to the Hub.

## Audit
- The Hub records an operation event:
  - `kind=secret_distribute`, `secret_kind=backup_age_identity`, `secret_name=<key_name>`, `target_agent=<agent_id>`
- No secret payload bytes are logged.

## Revocation (future)
This change focuses on distribution. Revocation / leasing can be layered later if needed:
- delete secret from Agent scope,
- trigger snapshot update,
- Agent removes the local entry on next snapshot apply.

