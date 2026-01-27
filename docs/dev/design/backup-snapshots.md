# 备份数据管理（Snapshots / Artifacts）设计与实现说明

本文档描述 Bastion 的“备份数据管理”子系统（snapshot 索引、删除队列、保留策略）的设计与实现要点。

用户侧使用方式请优先参考用户手册：`/user/backup-snapshots`。

目标能力：

- 查看历史成功备份产生的“备份数据（Snapshot/Artifact）”
- 手动删除备份数据（支持批量、可重试、可观测）
- 配置并执行保留策略（keep last N / keep days 等，服务端强制执行）
- 在多节点场景下正确地在“数据实际所在节点”执行删除（Hub/Agent）

> 设计原则：把“成功运行的备份产物”作为一等公民资源（snapshot），与“运行记录 run”分离，但保持可追溯关联。

---

## 1. 背景与现状

当前系统有：

- `runs`：运行记录（成功/失败/进度/摘要/错误等）
- “不完整运行清理”（`incomplete_cleanup_tasks` + UI）：用于清理失败/中断/不完整的 target run（例如 WebDAV 不可达导致清理失败会重试/可忽略/可查看事件）
- “备份数据（Snapshot）管理”（已实现）：`run_artifacts`（索引）、`artifact_delete_tasks`（删除队列 + 事件）、`snapshot_retention`（保留策略执行）

工程上可复用的模式：

- 任务队列 + 状态机 + 事件日志 + 退避重试（incomplete cleanup 已实现）
- 多节点上下文（Hub/Agent）
- `runs.target_snapshot_json` 已保存“运行时的目标快照”（用于维护流程不受 job spec 后续变更影响）

---

## 2. 术语

- **Run（运行记录）**：一次作业执行的过程与结果（status/progress/summary/error）。
- **Snapshot / Artifact（备份数据/备份点）**：某次成功 Run 在目标存储中落盘后的备份数据集合（通常对应 `job_id/run_id/` 目录）。
- **Target（目标存储）**：目前支持 `local_dir` / `webdav`（未来可扩展 S3 等）。
- **Node（节点）**：Hub 或 Agent。对于 `local_dir`，数据位于执行节点的本地文件系统；对于 `webdav`，数据位于远端 WebDAV（但“执行删除”的节点取决于凭据/网络可达性，通常为 Hub，或为具备 secret 的指定节点）。
- **Pinned（固定/保护）**：被标记为 pinned 的 snapshot 不会被保留策略删除；手动删除也需要更强确认/权限。

---

## 3. 目标与非目标

### 目标

1) 让“成功备份的数据”可被索引、列表展示、筛选与审计。

2) 删除必须是：

- 异步后台执行（避免 HTTP 超时）
- 幂等（重复请求不产生副作用；目标已不存在视为成功）
- 可观测（任务状态 + 尝试次数 + 最近错误 + 事件流）
- 可控（重试/忽略/放弃策略）

3) 保留策略必须是服务端强制执行的（避免只依赖前端），支持：

- keep last N（保留最近 N 份）
- keep days（保留最近 N 天）
- pinned 排除
- 预览/模拟（用户确认后执行）
- 安全阀：单次/单日最多删除数量（防止误配置删库）

4) 多节点正确性：

- “恢复到某个 Agent 的本地文件系统”已经存在，此处同样要求“删除 local_dir 的快照”由对应 Agent 执行。
- Hub 负责调度与观测，Agent 负责执行并回传结果/事件。

### 非目标（初期可不做）

- 面向对象存储（S3/OSS）完整实现（仅在接口层预留扩展点）
- 复杂 GFS（Grandfather-Father-Son）策略（可作为后续增强）
- 自动跨 target 复制、归档分层（后续再做）

---

## 4. 核心架构：把 Snapshot 当作一等公民

### 4.1 Snapshot 索引：Hub DB 作为 Source of Truth

建议在 Hub DB 引入 `run_artifacts`（或 `backup_snapshots`）表，将每个成功 run 的产物作为资源索引记录：

- 以 `run_id` 为主键（天然与 run 关联）
- 存储当时的 `target_snapshot_json`（避免 job spec 修改导致后续无法定位/删除旧数据）
- 存储必要的统计信息（大小、文件/目录数量、format），用于 UI 展示和保留策略计算

存储端仍以既有格式为准（`manifest.json` / `entries.jsonl.zst` / `complete.json` + payload）。

### 4.2 删除：独立的后台任务队列（可复用 incomplete cleanup 模式）

新增 `artifact_delete_tasks` + `artifact_delete_events`：

- 删除由任务驱动，支持重试/退避/忽略/放弃（abandoned）
- 与 UI 联动：用户可查看删除任务状态、错误分类、事件日志
- 删除完成后 `run_artifacts.status` 变为 `deleted`（或 `missing`）

### 4.3 保留策略：服务端定时执行 + 可预览

新增 retention loop（Hub 执行）：

- 读取每个 job 的 retention 配置（或默认值）
- 计算应保留/应删除集合（排除 pinned）
- 批量 enqueue 删除任务（受安全阀限制）

并提供 `preview` API 返回“将删除哪些、原因是什么”。

---

## 5. 数据模型（DB 设计）

### 5.1 表：`run_artifacts`（备份数据索引）

建议字段（SQLite）：

- `run_id TEXT PRIMARY KEY`（FK `runs(id)`）
- `job_id TEXT NOT NULL`
- `node_id TEXT NOT NULL`（数据所在节点；local_dir=执行节点）
- `target_type TEXT NOT NULL`（`local_dir`/`webdav`/未来扩展）
- `target_snapshot_json TEXT NOT NULL`
- `artifact_format TEXT NOT NULL`（`archive_v1`/`raw_tree_v1`）
- `status TEXT NOT NULL`：
  - `present`（可用）
  - `deleting`（删除中）
  - `deleted`（已删除）
  - `missing`（目标已不存在/被外部删除；可视为已清理）
  - `error`（处于错误/需要人工处理的状态）
- `created_at INTEGER NOT NULL`（建议=run.started_at）
- `ended_at INTEGER NOT NULL`（建议=run.ended_at）
- `pinned_at INTEGER`、`pinned_by_user_id INTEGER`
- `deleted_at INTEGER`、`deleted_by_user_id INTEGER`、`delete_reason TEXT`
- 统计信息（用于列表/策略/趋势）：
  - `source_files INTEGER`、`source_dirs INTEGER`、`source_bytes INTEGER`
  - `transfer_bytes INTEGER`（实际写入目标的数据量）
- 观测字段（用于 UI 与运维）：
  - `last_error_kind TEXT`、`last_error TEXT`
  - `last_attempt_at INTEGER`

索引建议：

- `idx_run_artifacts_job_status (job_id, status)`
- `idx_run_artifacts_job_ended (job_id, ended_at)`
- `idx_run_artifacts_node_status (node_id, status)`
- `idx_run_artifacts_pinned (pinned_at)`

> 注意：现有 `runs` 表已有 `target_snapshot_json` 与 `summary_json/progress_json`；`run_artifacts` 冗余一份快照是为了避免未来“run 表裁剪/归档”导致无法管理备份数据。

### 5.2 表：`artifact_delete_tasks`（删除任务队列）

可直接对齐 `incomplete_cleanup_tasks` 的字段与状态机：

- `run_id TEXT PRIMARY KEY`
- `job_id TEXT NOT NULL`
- `node_id TEXT NOT NULL`
- `target_type TEXT NOT NULL`
- `target_snapshot_json TEXT NOT NULL`
- `status TEXT NOT NULL`：
  - `queued | running | retrying | blocked | abandoned | done | ignored`
- `attempts INTEGER NOT NULL`
- `created_at/updated_at INTEGER NOT NULL`
- `last_attempt_at INTEGER`
- `next_attempt_at INTEGER NOT NULL`
- `last_error_kind TEXT`
- `last_error TEXT`
- `ignored_at INTEGER`
- `ignored_by_user_id INTEGER`
- `ignore_reason TEXT`

索引建议：

- `idx_artifact_delete_tasks_status_next_attempt (status, next_attempt_at)`
- `idx_artifact_delete_tasks_job_id (job_id)`
- `idx_artifact_delete_tasks_node_id (node_id)`

### 5.3 表：`artifact_delete_events`（删除任务事件）

对齐 `incomplete_cleanup_events`：

- `run_id TEXT NOT NULL`
- `seq INTEGER NOT NULL`
- `ts INTEGER NOT NULL`
- `level TEXT NOT NULL`
- `kind TEXT NOT NULL`
- `message TEXT NOT NULL`
- `fields_json TEXT`
- `PRIMARY KEY (run_id, seq)`
- `FOREIGN KEY (run_id) REFERENCES artifact_delete_tasks(run_id) ON DELETE CASCADE`

### 5.4 Retention 配置存储（两层）

建议两层配置：

1) 全局默认（Hub 设置，供新 job 继承）：放在 settings/hub_runtime_config（按现有模式）。
2) job 级覆盖：推荐放在 `jobs.spec_json`（更“配置即代码”），例如：

```json
{
  "v": 1,
  "type": "filesystem",
  "...": "...",
  "retention": {
    "enabled": true,
    "keep_last": 7,
    "keep_days": 30,
    "max_delete_per_day": 50
  }
}
```

> 规则：最终保留集合 = keep_last 与 keep_days 的并集；`pinned` 永不删除。

---

## 6. 运行时流程

### 6.1 写入索引（成功 run 完成时）

当 run 变为 `success` 时（Hub 或 Agent 完成回报后）：

1) 从 `runs.target_snapshot_json` 读取目标快照
2) 从 `runs.summary_json` / `manifest.json` / 进度统计中提取：
   - format（archive/raw_tree）
   - entries_count / files/dirs/bytes（尽可能统一口径）
3) upsert `run_artifacts`：
   - `status = present`
   - `node_id/target_type/target_snapshot_json`
   - `source_* / transfer_bytes`

### 6.2 删除（手动/保留策略触发）

删除流程：

1) API 收到删除请求：
   - 校验 run_artifacts 存在且 `status=present`（或允许 error/missing）
   - 若 pinned：需要更高确认（或禁止）
   - 若该 snapshot 正被 restore/verify 引用：默认禁止（需要 force）
   - 写入/更新 `artifact_delete_tasks`（queued）
   - `run_artifacts.status = deleting`
   - 记录 `artifact_delete_events`（kind=queued）
2) 删除 worker claim 任务并执行：
   - local_dir：在“数据所在节点（Agent）”执行删除
   - webdav：在“具备 secret/网络可达的节点”执行删除（默认 Hub）
3) 删除成功：
   - 标记 task `done`
   - `run_artifacts.status = deleted`，写 `deleted_at/by/reason`
4) 删除失败：
   - 分类错误（config/auth/network/http/unknown）
   - 根据策略进入 retrying/blocked/abandoned
   - 写事件与 last_error，UI 可见

> 幂等：目标不存在（404/NotFound）视为成功；重复删除请求不应导致重复任务。

### 6.3 保留策略执行（Hub）

周期性运行（建议每小时/每天）：

1) 遍历启用 retention 的 job（或所有 job 但使用默认值）
2) 列出该 job 的 snapshots：`status=present` 且 `pinned_at IS NULL`
3) 计算保留集合：
   - keep_last：按 `ended_at DESC` 取前 N
   - keep_days：保留 `ended_at >= now - days*86400`
   - keep = union
4) 其余进入删除候选，受 `max_delete_per_day`（或每轮限制）约束
5) 为每个候选 enqueue 删除任务（reason=retention）

同时提供 `preview` API：返回 keep/delete 及原因，供 UI 展示确认。

---

## 7. 多节点与协议（关键点）

### 7.1 问题：local_dir 数据在 Agent，本地删除必须由 Agent 执行

Hub 无法直接访问 Agent 的 `base_dir`，因此删除需要跨节点执行。

推荐方案：

- 在 agent 协议中新增一种“维护任务”（例如 `ArtifactDeleteTask`）：
  - Hub 下发：run_id + target_snapshot_json（或解析后的参数）
  - Agent 执行本地删除，并回传事件与最终结果
- Hub 侧持久化：
  - task 状态机（artifact_delete_tasks）
  - 事件日志（artifact_delete_events）

这样可实现：

- Agent 离线：任务保持 queued/retrying；Agent 上线后再执行
- UI 可观察：能看到“等待节点上线/网络错误/权限错误”等

### 7.2 webdav 删除：执行节点与凭据作用域

目前 WebDAV secret 是 node-scoped（参见 `docs/user/storage.md` 与 secret 分发能力）。

因此：

- 删除 webdav 快照应由“拥有该 secret 且网络可达”的节点执行
- 推荐默认：Hub 执行（Hub 通常持有 secret）
- 若未来支持“由 Agent 执行 webdav 删除”，需要确保 secret 已分发到该 Agent

---

## 8. HTTP API（建议）

以“Job → 备份数据（Snapshots）”为主入口：

### 列表

- `GET /api/jobs/:job_id/snapshots`
  - query：`status[]`、`pinned`、`from/to`、`cursor/limit`
  - 返回：列表项包含 ended_at/size/files/dirs/format/target/status/pinned

### 详情

- `GET /api/jobs/:job_id/snapshots/:run_id`
  - 返回：run_artifacts + 删除任务状态 + 最近事件

### 删除（单个/批量）

- `POST /api/jobs/:job_id/snapshots/:run_id/delete`
- `POST /api/jobs/:job_id/snapshots/delete`
  - body：`run_ids: []`，`force: bool`，`reason`

### 固定/取消固定

- `POST /api/jobs/:job_id/snapshots/:run_id/pin`
- `POST /api/jobs/:job_id/snapshots/:run_id/unpin`

### 保留策略

- `GET /api/jobs/:job_id/retention`
- `PUT /api/jobs/:job_id/retention`
- `POST /api/jobs/:job_id/retention/preview`
- `POST /api/jobs/:job_id/retention/apply`

---

## 9. Web UI（独立页面）

### 9.1 页面结构

建议新增独立页面（而非 Modal）：

- 路由：`/jobs/:job_id/snapshots`（或 `/jobs/:job_id/backups`）
- 入口：Jobs 列表/详情页中提供“备份数据”入口，与“运行记录”并列

### 9.2 列表信息（桌面/移动）

列表建议字段：

- 时间：结束时间（相对时间 + tooltip 绝对时间）
- 大小：源数据大小、传输大小（如可用）
- 规模：文件数/目录数
- 格式：archive_v1/raw_tree_v1
- 目标：产品化展示（例如“本地目录 /data/backup”或“WebDAV /backup”）
- 状态：中文（可用/删除中/已删除/缺失/错误）
- 固定：pin 图标
- 操作：恢复、删除、查看删除日志

功能：

- 多选批量删除
- 过滤：状态、时间范围、固定、format、节点（可选）
- 删除确认：展示将删除的条目清单；若 pinned/引用中强提示

### 9.3 删除任务可观测性

在 snapshot 列表中对“删除中/失败”提供入口：

- 展示 attempts、next_attempt、last_error_kind + 摘要
- 点击可打开“删除日志”抽屉/弹窗，展示 `artifact_delete_events`
- 提供按钮：立即重试 / 忽略（与不完整清理页面一致的交互语义）

### 9.4 保留策略配置 UI

在 Job 编辑页新增“保留策略”分组：

- enabled 开关
- keep_last、keep_days、max_delete_per_day
- “预览将删除哪些”按钮（调用 preview API）
- “立即执行清理”按钮（调用 apply）

---

## 10. 与现有 run retention 的关系（重要）

当前存在 `run_retention_days` 定时删除 `runs` 的逻辑。

引入 snapshots 后，必须避免出现：

- **runs 被删除，但备份数据仍存在且无法管理/恢复（孤儿数据）**

建议调整 run retention 的策略：

- 仅清理不再需要的 run：
  - 没有关联 `run_artifacts`，或 `run_artifacts.status IN (deleted, missing)`
  - 且 ended_at < cutoff

这样可以保证：

- 有备份数据的成功 run 记录不会被误删
- 删除备份数据后，run 记录可以按现有 retention 再被裁剪

---

## 11. 安全性与健壮性建议

- local_dir 删除前做“路径安全检查”：
  - 只能删除形如 `base_dir/job_id/run_id/` 的目录
  - 并做 “looks_like bastion”（manifest/entries/complete/payload.part*）校验，避免误删用户目录
- webdav 删除：
  - DELETE collection 时注意尾随 `/` 的兼容性
  - 对不支持递归删除的服务端，准备 fallback（PROPFIND 枚举后逐个 DELETE）
- 并发与限流：
  - 单节点同时删除任务数量限制（避免打满 IO/网络）
  - retention 删除数量限制（max_delete_per_day）
- 与 restore/verify 的互斥：
  - snapshot 被引用时禁止删除（默认）
- 审计：
  - 记录 deleted_by/原因，便于追踪误删

---

## 12. 测试计划

后端：

- retention 选择算法单测（keep_last/keep_days/union/pinned）
- 删除任务状态机集成测试（queued→running→done；retry/backoff；ignored）
- local_dir 删除安全测试（拒绝删除非 bastion 路径）
- webdav 删除适配测试（404 幂等、错误分类）

前端：

- snapshots 列表渲染与筛选
- 批量删除确认与状态刷新
- preview 结果渲染（keep/delete 原因）

---

## 13. 推荐实施顺序（里程碑）

1) **索引与只读列表**
   - DB：run_artifacts
   - run success 写入索引（upsert）
   - API：list/get snapshots
   - UI：独立页面只读展示

2) **删除任务 + 可观测**
   - DB：artifact_delete_tasks/events
   - 删除 worker（Hub 先支持 webdav；local_dir 需要 Agent 执行）
   - UI：删除、状态、事件日志、重试/忽略

3) **Pinned（固定保护）**
   - API/UI：pin/unpin
   - 删除/retention 排除 pinned

4) **Retention 策略**
   - job spec + 全局默认
   - preview + apply
   - retention loop 定时执行

5) **作业删除的级联删除**
   - 删除 job 时允许用户选择是否级联删除 snapshots（enqueue 删除任务）

6) **扩展点**
   - 支持更多 target（S3 等）：补充 target snapshot 与 delete adapter
