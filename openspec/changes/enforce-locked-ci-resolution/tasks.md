## 1. Reproduce And Repair

- [x] 1.1 Inspect failed platform/contract logs and compare the unlocked main script.
- [x] 1.2 Reproduce the lockfile failure with Rust 1.92.0.
- [x] 1.3 Normalize the lockfile without changing package versions.
- [x] 1.4 Require locked resolution in the main CI script.
- [x] 1.5 Verify locked resolution, diff checks, OpenSpec, and full CI availability.

## Validation

- Before normalization, `cargo update --workspace --locked` failed with the same lockfile-update error as CI.
- `cargo update --workspace --offline` changed only the ambiguous phf_macros dependency reference from `syn` to `syn 2.0.111`, locking zero new package versions.
- After normalization, `cargo update --workspace --locked --offline` passed.
- `bash -n scripts/ci.sh`, `git diff --check`, and OpenSpec strict validation passed.
- Full local CI passed the secrets scan (1,354 commits; no leaks) and changelog check, then stopped in the pre-existing changelog tests because this environment's awk lacks the required syntax. The environment cannot install gawk through apt; full application and cross-platform checks remain delegated to GitHub CI.
