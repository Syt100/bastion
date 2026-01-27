# 存储（WebDAV 凭据）

Bastion 当前支持 WebDAV 作为远端存储 target。

WebDAV 凭据会作为加密 secrets 存储在 Hub 数据库中，并且是 **node-scoped** 的：

- Hub 的 secrets 使用保留 node id：`hub`
- 每个 agent 有自己独立的 secret 命名空间

这意味着：

- 在某个 agent 上运行的 job 如果引用了某个 WebDAV credential name，则该 agent 的 secret 里必须存在同名凭据。
- 仅在 Hub 上创建凭据并不足以让 agent-run job 使用它。

## 管理 WebDAV 凭据

在 Web UI：

- **Settings → Storage**（默认处于 Hub context）

Storage 设置会在一个 node context 下显示（`hub` 或某个 agent）。你可以：

- 按名称创建/编辑/删除凭据
- 查看更新时间
- 复制凭据名用于 job 配置

## 将 WebDAV 凭据分发到 Agents

把 Hub 的某个凭据复制到多个 agents：

1. 确保该凭据已存在于 **Hub**（node `hub`）
2. 点击 **Distribute**
3. 选择目标 agents（按 labels 或直接填写 node IDs）
4. （可选）勾选 **overwrite**
5. 先 preview，再创建 bulk operation

进度与按 agent 的结果可在以下位置查看：

- **Settings → Bulk operations**

## WebDAV 凭据的使用场景

- **Job targets**：当 target 类型选择 **WebDAV** 时，需要填写 credential name。
- **Restore to WebDAV**：restore 使用执行 node 的 WebDAV secret 命名空间；如果你把 restore 放到 agent 上执行，可能需要先把同名凭据分发到该 agent。

另见：[恢复与校验](/zh/user/restore-verify)。

