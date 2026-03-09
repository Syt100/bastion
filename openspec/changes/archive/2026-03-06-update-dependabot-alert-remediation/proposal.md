# Change: Remediate open Dependabot alerts across Rust and Node lockfiles

## Why
- Dependabot currently reports open vulnerabilities in both Rust and npm dependency graphs.
- Several alerts are runtime-scope and should be remediated to keep default-branch security posture healthy.
- We need a repeatable dependency hygiene pattern that fixes known CVEs without destabilizing build workflows.

## What Changes
- Rust workspace: update vulnerable dependency resolution for `time` and `bytes`, and reduce unused SQLx attack surface that pulls unnecessary crypto deps.
- UI lockfile: pin vulnerable `lodash`/`lodash-es` transitive versions to patched releases using npm override strategy.
- Docs lockfile: attempt patched `esbuild` override path, and keep only verified-compatible remediation.
- Validate with local CI and GitHub security scans.

## Impact
- Affected areas: workspace `Cargo.toml`/`Cargo.lock`, `ui/package.json` + lockfile, optional docs lockfile override path.
- Security posture: targeted closure of current Dependabot alerts with minimal functional change.
- Compatibility: production code unchanged; dependency graph and lockfiles updated with regression checks.
