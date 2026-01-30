# 用户手册

Bastion 是面向小规模部署（个人/家庭/小团队）的自托管备份编排器。

如果你刚开始接触 Bastion，建议先读： [概念与术语](/zh/user/concepts)。

## 组成

- **Hub**：主服务（HTTP API + Web UI）。使用 SQLite 存储元数据，并管理加密的 secrets。
- **客户端（Agent）**（可选）：连接到 Hub，在另一台机器上执行备份任务。

## 典型使用流程

1. [启动 Hub 并完成首次初始化](/zh/user/getting-started)。
2. （可选）[接入客户端](/zh/user/agents) 来做多节点备份。
3. 创建 [备份任务](/zh/user/jobs) 并运行。
4. 查看 [运行记录](/zh/user/runs)，并通过 [恢复与校验](/zh/user/restore-verify) 做恢复与完整性校验。
5. 在 [备份快照](/zh/user/backup-snapshots) 中管理备份产物（固定/删除/保留策略）。
6. 如果需要远端存储，配置 [存储（WebDAV）](/zh/user/storage)。
7. （可选）配置 [通知](/zh/user/operations/notifications)。

## 参考

- [批量操作](/zh/user/bulk-operations)（标签、同步配置、分发 WebDAV 凭据、部署任务）
- [运维](/zh/user/operations/defaults)（默认行为 / 升级与回滚 / 反向代理 / 运行时配置 / 日志 / 可观测性）
- [配方](/zh/user/recipes/vaultwarden)
