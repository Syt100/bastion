# 维护（incomplete cleanup）

Bastion 在运行过程中会产生临时的“staging”数据。如果某次 run 失败、被中断，或 Hub/Agent 崩溃，可能会留下部分临时数据。

为避免磁盘占用持续增长，Bastion 提供 **incomplete cleanup** 任务队列，用于自动清理“足够旧”的不完整 runs。

在 Web UI：

- **Settings → Maintenance → Cleanup**

## 清理对象

incomplete cleanup 任务会针对 **非 success** 的 runs（failed/rejected）且早于截止时间的记录。

根据该 run 的 target 类型，清理可能包含：

- 删除本地 staging 目录，和/或
- 清理远端的部分输出（例如未完成的 WebDAV 上传）

它与 snapshot 删除/保留策略是两套机制（见 [备份快照](/zh/user/backup-snapshots)）。

## 状态含义

- **queued**：等待执行
- **running**：执行中
- **retrying**：之前失败，稍后重试
- **blocked**：无法自动推进（需要用户处理或环境修复）
- **abandoned**：重试次数过多/任务过旧，已放弃
- **done**：清理成功
- **ignored**：用户显式忽略

## UI 可用操作

- **Retry now**：立即安排重试（通常在你修复根因后使用）
- **Ignore**：停止重试该任务（例如你已手动清理，或接受残留）
- **Unignore**：把 ignored 的任务重新放回队列
- 打开任务查看 **事件日志** 与最近错误详情

## 配置

截止时间由 **Incomplete cleanup days** 控制：

- Web UI：**Settings → Runtime config**
- CLI/env：`--incomplete-cleanup-days` / `BASTION_INCOMPLETE_CLEANUP_DAYS`

说明：

- 默认 `7` 天
- 设为 `0` 会禁用 incomplete cleanup 循环（需要你自行手动清理）

