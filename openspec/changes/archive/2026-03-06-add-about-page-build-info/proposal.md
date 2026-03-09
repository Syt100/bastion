## Why
Users need an easy way to identify what version of Bastion they are running and when it was built, especially when debugging issues across different deployments.

## What Changes
- Add an authenticated "About" page in the Web UI.
- Display Hub (backend) version + build time.
- Display Web UI version + build time.

## Impact
- No breaking API changes (only additive fields).
- Local development builds and source builds without git metadata must continue to work (fields may fall back to `unknown`).
