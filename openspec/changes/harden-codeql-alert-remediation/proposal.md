# Change: Remediate CodeQL secret-hygiene alerts

## Why
- Code scanning currently reports repeated `hard-coded-cryptographic-value` and `cleartext-logging` alerts.
- Most findings are in tests, but they still break security quality gates and hide true positives.
- We need a durable, low-noise pattern for test credentials and secret assertions.

## What Changes
- Replace hard-coded test password/keypack literals with runtime-generated passphrases.
- Refactor sensitive test assertions to avoid printing secret material in failure output.
- Add shared test helper patterns so new tests do not reintroduce these alerts.
- Re-run CI and code-scanning to confirm alert reduction.

## Impact
- Affected areas: `bastion-http`, `bastion-storage`, `bastion-engine`, `bastion-backup`
- Security posture: reduced secret exposure risk in logs and stronger code-scanning signal quality
- Compatibility: no runtime behavior changes for production code paths
