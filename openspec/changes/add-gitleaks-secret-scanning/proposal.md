# Change: Add gitleaks secret scanning

## Why
Secrets (API keys, tokens, private keys, passwords) are easy to accidentally commit.
Even a single leaked credential can turn into a security incident and requires key rotation and audit.

We want a fast, automated guardrail that fails CI when likely secrets are detected in the Git history or working tree.

## What Changes
- Add a `gitleaks` scan step to the repo CI scripts:
  - `scripts/ci.sh`
  - `scripts/ci.ps1`
- Prefer running gitleaks with redaction enabled so potential secrets are not printed in plaintext logs.
- If `gitleaks` is not installed, CI scripts will attempt to install a pinned version via `go install` (when `go` is available).

## Impact
- Affected specs: `dev-workflow`
- Affected code: `scripts/ci.sh`, `scripts/ci.ps1`

## Non-Goals
- Adding pre-commit hooks.
- Introducing a new CI provider configuration (GitHub Actions, etc.) in this change.

