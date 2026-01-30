# 批量操作（Bulk operations）

批量操作是对一组客户端执行的、可追踪的异步操作。

适用于：批量更新标签、分发凭据、同步配置、或把某个任务克隆到多台机器等。

## 概念

- **Bulk operation**：一个顶层请求（kind + selector + payload）。
- **Bulk operation item**：按客户端维度的执行记录。

每个 item 会记录：

- 状态（`queued`、`running`、`success`、`failed`、`canceled`）
- 尝试次数（attempts）
- 最近一次错误类型/信息（或成功说明）

## UI

- 查看与管理：**设置 → 批量操作**
  - 打开某个操作查看按客户端的执行项
  - **重试失败项**：只重试失败的执行项
  - **取消**：取消仍在排队的执行项（正在运行的不会被强制中断）

很多批量操作在创建前支持 **preview**（预览/干跑）来展示计划。

## 当前支持的 kinds

### `agent_labels_add` / `agent_labels_remove`

为选中的客户端批量添加/删除标签。

入口：

- **客户端** → **批量编辑标签**

### `sync_config_now`

请求客户端拉取/应用最新的配置快照（config snapshot）。

入口：

- **客户端** → **同步配置**

### `webdav_secret_distribute`

把 Hub 的某个 WebDAV 凭据复制到选中的客户端。

说明：

- 凭据是按节点隔离的：在某个客户端上运行的任务如果引用了 WebDAV 凭据名称，则该客户端必须存在同名凭据。
- 此操作可以选择 **overwrite** 或 **skip**（当客户端上已存在同名凭据时）。

入口：

- **设置 → 存储**（Hub 节点）→ **分发**

### `job_deploy`

把一个已有任务克隆到选中的客户端。

说明：

- 使用命名模板（默认 `{name} ({node})`），遇到冲突会自动加后缀。
- 会做按 node 的校验（例如：缺少 node-scoped secrets）。

入口：

- **备份任务** → 选择一个任务 → **部署到节点**

## API（可选参考）

Hub 暴露的接口：

- `POST /api/bulk-operations` — 创建批量操作
- `POST /api/bulk-operations/preview` — 预览（仅支持部分 kinds）
- `GET /api/bulk-operations` — 列表
- `GET /api/bulk-operations/{id}` — 详情（含 items）
- `POST /api/bulk-operations/{id}/retry-failed` — 重试失败项
- `POST /api/bulk-operations/{id}/cancel` — 取消
