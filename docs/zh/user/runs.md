# Runs（运行记录）

**Run** 是 job 的一次执行。Run 会记录状态、时间、进度以及事件日志。

## 在哪里查看 Runs

在 Web UI：

- **Jobs** → 选择某个 job → **Runs**
- 点击某个 run 打开 **run 详情页**

## 状态含义

- **queued**：已接受请求，等待执行
- **running**：正在所选 node 上执行
- **success**：成功结束（可能产出 snapshot）
- **failed**：失败结束
- **rejected**：由于 overlap policy 被拒绝（例如 overlap=reject 且已有 run 在 running）

## Run 详情页（可以做什么）

Run 详情页包含：

- **Summary**：状态、时间、基础指标，以及本次使用的 source/target
- **Live events**：run 执行期间的增量事件/进度（WebSocket）
- **Operations**：从该 run 发起的 restore/verify 操作

对于成功 run，你还可以发起：

- **Restore**（恢复到目的地）
- **Verify**（恢复演练 + 哈希校验）

见：[恢复与校验](/zh/user/restore-verify)。

## Run 历史保留说明（Run retention）

Hub 会按 **Run retention days** 自动清理旧的 run 记录。

重要行为：

- run 清理是 **snapshot-aware** 的：只要该成功 run 仍然有“存活”的 snapshot（present/deleting/error），该 run 记录就会被保留。
- 当你删除 snapshot（或被 retention 完全删除）后，对应的 run 记录会在超过保留截止时间后被清理。

见：[运行时配置](/zh/user/operations/runtime-config)。

