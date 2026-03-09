# Change: Refactor job_spec module structure

## Why
`crates/bastion-core/src/job_spec.rs` currently mixes type definitions and validation/parsing logic. Splitting it into focused submodules improves maintainability and keeps the public API (`bastion_core::job_spec::*`) easy to navigate.

## What Changes
- Convert `job_spec.rs` into a folder module (`job_spec/mod.rs`)
- Split type definitions and validation/parsing logic into focused submodules
- Preserve existing public API and behavior (`job_spec::{types, parse_value, validate, ...}`)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-core/src/job_spec.rs` and new `crates/bastion-core/src/job_spec/` submodules

## Compatibility / Non-Goals
- No behavior changes intended for job spec parsing, validation rules, or serialization format.

