# 架构

## 高层组件

- **Hub**（单一二进制）：HTTP API + Web UI、调度/队列、SQLite 元数据存储、加密 secrets。
- **Agent**（Hub 子命令）：通过 WebSocket 连接 Hub，在远端机器上执行 jobs。
- **Web UI**（`ui/`）：Vue 3 + Vite 的单页应用。

## 数据模型（概念层）

- **Jobs**：要执行什么、在哪里执行、何时执行（schedule + timezone + overlap policy）。
- **Runs**：执行记录（状态/进度/摘要/事件）。
- **Run artifacts（snapshots）**：成功 run 的备份产物索引记录。
- **Secrets**：加密后存放在 Hub 数据库中；很多是 **node-scoped**（Hub vs 每个 Agent）。

## 后台 workers（示例）

- **Artifact delete queue**：快照删除队列（异步、可重试、带事件日志）。
- **Snapshot retention loop**：按 job 保留策略（keep last / keep days）执行删除，并受安全阀限制。
- **Incomplete cleanup**：清理失败/中断 run 的 staging 目录与残留数据。
- **Run retention**：清理旧 run 历史，但会保留仍有“存活快照”的 runs。
- **Notifications loop**：run 结束后发送队列中的 WeCom/email 通知。

用户可见行为与 UI 入口请参考：[用户手册](/zh/user/)。

