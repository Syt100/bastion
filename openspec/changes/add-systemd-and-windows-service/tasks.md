## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for systemd + Windows Service support
- [x] 1.2 Run `openspec validate add-systemd-and-windows-service --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Linux: add `packaging/linux/bastion.service` (+ env template) and include in `.deb/.rpm`
- [x] 2.2 Linux: handle `SIGTERM` for graceful shutdown
- [x] 2.3 Windows: add service entrypoint (`bastion.exe service run`) and SCM stop handling
- [x] 2.4 Windows: update WiX MSI to install the service (but not start it)
- [x] 2.5 Docs: update install/run instructions (EN/ZH) to cover service start after install

## 3. Validation
- [x] 3.1 Run `bash scripts/ci.sh`
- [ ] 3.2 Validate release packaging via GitHub Actions (workflow_dispatch dry-run is OK)

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit
