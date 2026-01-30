# 运行时配置（Hub settings）

Bastion 在 Web UI 中提供一组 Hub 的运行时配置项。

在 Web UI：

- **设置 → 运行时配置**

## 生效值与保存值

运行时配置页面会同时展示：

- **生效值（Effective value）**：当前 Hub 实际在使用的值
- **已保存值（Saved value）**：保存到数据库中的值（当没有被 CLI/env 覆盖时，会在重启后生效）
- **来源（Source）**：生效值的来源（`cli`、`env`、`db`、`default`）

优先级（从高到低）：

1. 命令行参数（CLI）
2. 环境变量
3. 数据库保存值
4. 内置默认值

如果某个字段被 CLI/env 覆盖，那么你在 UI 里保存的不同值不会立刻改变生效值；只有移除覆盖并重启后才会生效。

## 配置项说明

### Hub timezone

新建任务的默认计划时区（IANA 时区名）：

- 例如：`UTC`、`Asia/Shanghai`、`America/Los_Angeles`
- 环境变量：`BASTION_HUB_TIMEZONE`

该设置不会修改已经存在的任务（它们已经有各自明确的计划时区）。

### Run retention days

数据库中运行记录的保留天数（默认：180）：

- 环境变量：`BASTION_RUN_RETENTION_DAYS`

说明：

- 运行记录清理与快照关联：只要该成功运行仍有“存活”的快照（present/deleting/error），对应的运行记录会被保留。

### Incomplete cleanup days

是否自动清理失败/中断导致的“未完整清理”的运行（默认：7）：

- 环境变量：`BASTION_INCOMPLETE_CLEANUP_DAYS`
- `0` 表示禁用 incomplete cleanup 循环

另见：[维护（incomplete cleanup）](/zh/user/operations/maintenance)。

### Logging

可通过运行时配置页面设置日志：

- Log filter：`BASTION_LOG` / `RUST_LOG`
- Log file：`BASTION_LOG_FILE`
- Rotation：`BASTION_LOG_ROTATION`（`daily|hourly|never`）
- Keep files：`BASTION_LOG_KEEP_FILES`

另见：[日志](/zh/user/operations/logging)。

### 默认快照保留（新建任务）

以下默认值会在你创建 **新任务** 时在任务编辑器中自动带出：

- enabled
- keep last / keep days
- max delete per tick / per day（安全阀）

说明：

- 修改默认值不会影响已经存在的任务。
- 该设置会被 UI 立即使用（不需要重启）。

## 重启提示

多数运行时配置字段是在 Hub 启动时加载的。

当你修改 Hub timezone / run retention / incomplete cleanup / logging 等配置后，建议重启 Hub 以应用新的生效值。
