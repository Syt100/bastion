# 恢复与校验（Restore / Verify）

Restore 与 Verify 都是从 **成功的 run** 发起的长耗时 **operation（操作）**。

## 发起操作

在 Web UI：

1. 打开某个 job 的 **Runs**，选择一个 **success** 的 run
2. 点击 **Restore** 或 **Verify**

操作会显示在 run 详情页中（状态 + 进度 + 事件日志）。

## Restore（恢复）

Restore 会读取 snapshot，并把文件写入到你选择的目的地。

### 目的地类型

#### 本地文件系统（Local filesystem）

- **Node**：在哪个 node 上执行恢复（Hub 或某个 Agent）
- **Destination directory**：该 node 上的目标目录

说明：

- 目录路径在所选 node 的文件系统中解释。
- **Browse** 需要所选 node 在线。

#### WebDAV

- **Base URL**：例如 `https://dav.example.com/backup-restore`
- **Secret name**：选择 WebDAV 凭据名
- **Prefix**：Base URL 下的目标子目录

重要：

- WebDAV 凭据是 **node-scoped** 的。只有当“执行 restore 的 node”具备对应 WebDAV secret 时，restore 才能成功。

见：[存储（WebDAV）](/zh/user/storage)。

### 冲突策略（Conflict policy）

当目的地已存在同名路径时：

- **overwrite**：覆盖已有文件
- **skip**：保留已有文件，跳过冲突项
- **fail**：遇到第一处冲突即失败退出

### 选择性恢复（可选）

你可以：

- 默认恢复全部内容，或
- 仅从 run entries 列表中选择部分文件/目录恢复

## Verify（校验 / 恢复演练）

Verify 是一套安全检查流程：

1. 拉取 snapshot
2. 恢复到一个 **临时目录**
3. 按快照索引做文件哈希校验
4. 适用时对 SQLite 文件做完整性检查

如果校验失败，operation 会标记为 **failed**，事件日志会包含部分示例错误。

## 多节点注意事项与当前限制

- **加密备份 + 恢复到 Agent**：当你把加密备份恢复到 agent 时，Hub 会在派发 restore 前自动确保该 agent 具备所需私钥。
- **Verify 当前在 Hub 上执行**。这意味着：
  - WebDAV target 的快照：只要 Hub 有对应 WebDAV secret，就可以校验。
  - Agent 上的本地目录快照：通常 **无法** 在 Hub 上做 verify（除非 Hub 能访问到该目录，例如共享挂载）。

如果你在多节点部署中非常依赖 Hub 侧的 verify 演练，建议优先使用 **WebDAV** 作为 target。

