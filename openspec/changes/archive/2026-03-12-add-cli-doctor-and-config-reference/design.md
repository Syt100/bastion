## Context
The CLI already exposes `bastion config` and `bastion doctor`, and the repository already generates a CLI reference page from the real clap command tree. The remaining gap is that operators still do not have a single generated reference for runtime-related flags and environment variables, especially for env-only knobs such as UI/docs asset overrides.

This change finishes the workflow by making configuration reference docs generation a first-class part of docgen and CI.

## Goals / Non-Goals

### Goals
- Keep `bastion config` and `bastion doctor` behavior aligned with the actual runtime config resolution path.
- Generate config/environment reference pages from real CLI definitions wherever possible.
- Include env-only runtime knobs that are not represented in clap but still affect shipped behavior.
- Make CI fail when generated reference outputs drift from code.

### Non-Goals
- Rework the Hub runtime-config Web UI data model.
- Introduce a generic documentation extraction framework across the whole repository.
- Cover build-only/internal environment variables that are not operator-facing runtime configuration.

## Decisions

### 1. Keep CLI reference and config reference as separate generated artifacts
The existing CLI reference remains a full command-tree dump. The new config reference focuses on configuration-related flags and environment variables so operators can scan effective knobs faster without reading the entire help tree.

### 2. Use clap as the primary source of truth
For CLI-backed configuration, docgen walks the localized clap command tree and extracts:
- command scope,
- flag names,
- env variable names,
- defaults,
- localized descriptions.

This keeps English and zh-CN docs aligned with the shipped binary and help translations.

### 3. Curate a small env-only supplement in docgen
Some runtime knobs are not represented in clap (for example `BASTION_UI_DIR`, `BASTION_DOCS_DIR`, `BASTION_LANG`, and the `RUST_LOG` fallback path). These are added as a small explicit supplement in docgen because there is no clap metadata to extract from.

### 4. Keep CI drift detection in the existing docgen check
`scripts/ci.sh` already runs docgen in check mode. Extending docgen to emit the config reference pages means the existing CI check naturally covers both CLI reference and config/env reference drift.

## Risks / Trade-offs
- The curated env-only supplement can drift if future env-only knobs are added without updating docgen.
- clap-derived descriptions are help-text oriented, so the generated config reference is intentionally concise rather than deeply tutorial.
- Splitting CLI reference and config reference improves discoverability, but it creates another generated page that contributors must keep in sync through docgen.
