## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for workspace dependency centralization
- [x] 1.2 Run `openspec validate refactor-workspace-dependencies --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Rust workspace - dependency management
- [x] 2.1 Add `[workspace.dependencies]` for common shared crates (Tokio/Axum/SQLx/Serde/etc.)
- [x] 2.2 Update member crates to use `workspace = true` for centralized dependencies
- [x] 2.3 Ensure `cargo test --workspace` passes
- [x] 2.4 Commit workspace dependency refactor changes (detailed message)
