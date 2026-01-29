# 客户端（Agents）

本页介绍客户端（Agent）的接入（Enrollment）以及在 Web UI 中的日常管理。

## 接入（Enrollment）

高层流程：

1. 在 Web UI 中：进入 **客户端** 页面，创建 **接入令牌**
2. 在目标机器上：运行 `bastion agent ... --enroll-token <token>` 完成接入

接入令牌具有时效（默认 1 小时），也可以配置使用次数上限。请将其视为敏感信息妥善保管。

示例：

```bash
./bastion agent \
  --hub-url http://127.0.0.1:9876 \
  --enroll-token <token> \
  --name "<friendly-name>"
```

说明：

- Agent 会把自己的接入身份信息存放在自身的数据目录中（`--data-dir` / `BASTION_DATA_DIR`）。
- 如果 Agent 已经接入过，则不需要 `--enroll-token`。

## Agent 状态与生命周期

Agent 可能处于以下状态：

- **online**：最近处于连接状态（Hub 会显示在线）
- **offline**：当前未连接（部分动作会排队，等它重新连接后再执行）
- **revoked**：已被显式吊销；应视为不可信/不再允许连接

在“客户端”页面打开某个 Agent 的详情，可以看到 config snapshot 状态与最近错误。

## 标签（Labels）

你可以给 Agent 添加任意标签（例如 `prod`、`cn`、`db`）。

标签的使用场景：

- **客户端列表筛选**：按标签过滤客户端（支持 AND/OR 模式）。
- **批量操作选择器**：批量操作按标签选择目标客户端。

常见标签模式：

- 环境：`prod`、`staging`、`dev`
- 地域：`cn`、`us`、`eu`
- 角色：`db`、`web`、`media`

## 配置同步（Config sync）

Hub 会为每个 Agent 生成一份“config snapshot”（包含该 Agent 需要的 jobs、secrets 以及与运行相关的配置）。
Agent 在线时会拉取并应用这份 snapshot。

在 Web UI（客户端页面）里，每个 Agent 都会显示：

- **Desired snapshot ID**：Hub 希望 Agent 下一次应用的 snapshot
- **Applied snapshot ID**：Agent 上一次上报已应用的 snapshot
- **Last error**：最近一次同步错误（类型/消息/时间戳）

可用动作：

- **Sync now**（单个 Agent）：请求该 Agent 立即同步
- **Sync config now**（批量）：对多个 Agent 发起同步请求（批量操作）

注意：

- 若 Agent **offline**，同步请求会被记录，待其重新连接后再投递。
- 批量同步与其他批量动作的进度可在 **Settings → Bulk operations** 中查看。

## 安全相关动作（轮换密钥 / 撤销）

### 轮换 Agent 密钥

轮换密钥会为同一个 Agent ID 生成新的凭据（agent_key）。

- UI 会只展示一次新的 key；你需要更新该 Agent 数据目录中的 `agent.json`，并重启 Agent。

### 撤销 Agent

Revoke 会在 Hub 侧将该 Agent 标记为 revoked；被吊销的 Agent 应当被视为已泄露/不可信。

如果你希望重新接入该机器，请把它当作新机器重新接入。
