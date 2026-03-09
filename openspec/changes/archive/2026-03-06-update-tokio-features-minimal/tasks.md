## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for minimizing Tokio feature flags and preventing `tokio/full`
- [x] 1.2 Run `openspec validate update-tokio-features-minimal --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Rust workspace - Tokio features
- [x] 2.1 Replace `tokio` `features = ["full"]` with an explicit minimal feature set in affected crates
- [x] 2.2 Ensure `cargo test --workspace` passes
- [x] 2.3 Add a CI check that fails if any crate declares `tokio` with the `full` feature
- [x] 2.4 Commit Tokio feature minimization changes (detailed message)
