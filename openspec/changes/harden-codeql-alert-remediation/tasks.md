## 1. Spec
- [x] 1.1 Add backend spec delta for code-scanning secret-hygiene remediation
- [x] 1.2 Run `openspec validate harden-codeql-alert-remediation --strict`

## 2. Hard-coded Credential Alert Remediation
- [x] 2.1 Replace hard-coded test passwords in `bastion-http`/`bastion-engine`/`bastion-storage` test code with runtime-generated passphrases
- [x] 2.2 Replace hard-coded keypack test passphrases with runtime-generated passphrases
- [x] 2.3 Keep test readability with small helper utilities where needed

## 3. Cleartext Logging Alert Remediation
- [x] 3.1 Refactor sensitive assertions in backup/restore tests to avoid formatting secret values
- [x] 3.2 Remove `{:?}` panic branches that could dump secret-bearing variants

## 4. Validation
- [x] 4.1 Run `cargo fmt --all`
- [x] 4.2 Run targeted Rust tests for touched crates/modules
- [x] 4.3 Run `bash scripts/ci.sh`
- [x] 4.4 Mark checklist complete
