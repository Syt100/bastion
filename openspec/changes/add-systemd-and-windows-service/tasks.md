## 1. Spec
- [ ] 1.1 Add `dev-workflow` spec delta for systemd + Windows Service support
- [ ] 1.2 Run `openspec validate add-systemd-and-windows-service --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Linux: add `packaging/linux/bastion.service` (+ env template) and include in `.deb/.rpm`
- [ ] 2.2 Linux: handle `SIGTERM` for graceful shutdown
- [ ] 2.3 Windows: add service entrypoint (`bastion.exe service run`) and SCM stop handling
- [ ] 2.4 Windows: update WiX MSI to install the service (but not start it)
- [ ] 2.5 Docs: update install/run instructions (EN/ZH) to cover service start after install

## 3. Validation
- [ ] 3.1 Run `bash scripts/ci.sh`
- [ ] 3.2 Validate release packaging via GitHub Actions (workflow_dispatch dry-run is OK)

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit
