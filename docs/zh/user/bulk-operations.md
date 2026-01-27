# 批量操作（Bulk operations）

批量操作是对一组 agents（nodes）执行的、可追踪的异步动作。

适用于：批量更新 labels、分发凭据、触发同步配置、或把某个 job 克隆到多台机器等。

## 概念

- **Bulk operation**：一个顶层请求（kind + selector + payload）。
- **Bulk operation item**：按 agent 维度的执行记录。

每个 item 会记录：

- 状态（`queued`、`running`、`success`、`failed`、`canceled`）
- 尝试次数（attempts）
- 最近一次错误类型/信息（或成功说明）

## UI

- 查看与管理：**Settings → Bulk operations**
  - 打开某个操作查看按 agent 的 items
  - **Retry failed**：只重试失败的 items
  - **Cancel**：取消仍在排队的 items（正在 running 的不会被强制中断）

很多批量操作在创建前支持 **preview**（预览/干跑）来展示计划。

## 当前支持的 kinds

### `agent_labels_add` / `agent_labels_remove`

为选中的 agents 批量添加/删除 labels。

入口：

- **Agents** → **Bulk labels**

### `sync_config_now`

请求 agents 拉取/应用最新的 config snapshot。

入口：

- **Agents** → **Sync config now**

### `webdav_secret_distribute`

把 Hub 的某个 WebDAV 凭据复制到选中的 agents。

说明：

- Secrets 是 node-scoped 的。一个在 agent 上运行的 job 如果引用了某个 WebDAV secret name，则该 agent 必须存在同名 secret。
- 此操作可以选择 **overwrite** 或 **skip**（当 agent 上已存在同名 secret 时）。

入口：

- **Settings → Storage**（Hub node）→ **Distribute**

### `job_deploy`

把一个已有 job 克隆到选中的 agents。

说明：

- 使用命名模板（默认 `{name} ({node})`），遇到冲突会自动加后缀。
- 会做按 node 的校验（例如：缺少 node-scoped secrets）。

入口：

- **Jobs** → 选择一个 job → **Deploy to nodes**

## API（可选参考）

Hub 暴露的接口：

- `POST /api/bulk-operations` — 创建批量操作
- `POST /api/bulk-operations/preview` — 预览（仅支持部分 kinds）
- `GET /api/bulk-operations` — 列表
- `GET /api/bulk-operations/{id}` — 详情（含 items）
- `POST /api/bulk-operations/{id}/retry-failed` — 重试失败项
- `POST /api/bulk-operations/{id}/cancel` — 取消

