## 1. Configuration

- [x] 1.1 Document the security-grouping requirements before implementation.
- [x] 1.2 Add dedicated minor/patch security groups to all four update entries.
- [x] 1.3 Validate YAML against the Dependabot JSON schema, check all four group scopes, verify existing configuration is preserved, and validate OpenSpec with `--strict`.
- [x] 1.4 Run the repository CI script and record any environment limitations.

## Validation Notes

- `git diff --check`: passed.
- YAML parsing and Dependabot JSON schema validation: passed.
- Existing configuration preservation and four security-group scope checks: passed.
- `openspec validate group-dependabot-security-updates --strict --no-interactive`: valid.
- `bash scripts/ci.sh`: blocked at the initial secrets scan because neither gitleaks nor Go is installed. Application build/tests were not reached; complete CI remains pending.
