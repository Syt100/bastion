# 概念与术语

本页用于解释 Bastion 与 Web UI 中的核心概念，便于你快速建立正确的“心智模型”。

## Hub、Agent 与 Node

- **Hub**：Bastion 主服务（HTTP API + Web UI + 调度/队列）。在 UI 和 API 中，Hub 使用特殊的 node id：`hub`。
- **Agent**（可选）：远端执行节点，通过 WebSocket 连接到 Hub，在另一台机器上执行作业。
- **Node**：泛指 Hub 或某个 Agent。一个 Job 只会在 **一个** node 上执行。

## Job、Run 与 Snapshot

- **Job（作业）**：一次备份的配置（source + target + schedule + retention + notifications）。
- **Run（运行）**：Job 的一次执行（queued → running → success/failed/rejected）。Run 有事件/日志，也可能产出备份数据。
- **Snapshot（快照）**：一次成功 run 产出的“已存储备份数据”（以及 Hub 中的索引记录）。
  - Snapshot 才是你在 UI 里“固定/删除/保留策略”的对象。
  - Snapshot 与 Run 分离：这样即使旧 run 记录被清理，备份数据仍然可以继续管理。

## Operations（Restore / Verify）

**Operation（操作）** 是从成功 run 发起的长耗时动作：

- **Restore（恢复）**：将快照中的全部或部分文件恢复到指定目的地（本地目录或 WebDAV）。
- **Verify（校验）**：一次“恢复演练”，恢复到临时目录并做哈希校验（适用时也会做 SQLite 完整性检查）。

Operation 有独立的进度与事件日志，会显示在 run 详情页中。

## Secrets（凭据）

Bastion 将凭据作为 **加密 secrets** 存储在 Hub 数据库中。

- **按 node 分域（node-scoped）**（重要）：很多 secret 是按 node 隔离的（Hub vs 每个 Agent）。
  - 例：WebDAV 凭据必须存在于“实际执行 WebDAV 上传/下载”的 node 上。
  - 需要时可用批量操作把 Hub 的 secret 分发到 agents。

## Bulk operations（批量操作）

**批量操作** 是对多个 agent 异步执行的动作（例如：批量加/删 label、批量同步配置、分发 WebDAV 凭据、批量部署作业）。

- 每个批量操作会包含按 agent 划分的 **items**（每个 item 有自己的状态与错误信息）。

