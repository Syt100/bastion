# 驱动平台开发流程

本文说明如何在现有的 registry + planner 架构下，新增备份**来源驱动**或**目标驱动**。

## 关键模块

- `crates/bastion-driver-api`：共享驱动契约与能力模型，包含
  `TargetDriver::open_reader` 与 `TargetRunReader` 读取契约。
- `crates/bastion-driver-registry`：驱动注册、目标生命周期（`open_writer`、`open_reader`、
  `cleanup_run`、`snapshot_redacted`）与内置适配。
- `crates/bastion-driver-registry/src/target_runtime.rs`：共享运行时映射工具，将 target
  规格/解析结果映射为 `(DriverId, target_config)`。
- `crates/bastion-core/src/execution_planner.rs`：基于能力协商的确定性规划器。
- `crates/bastion-engine/src/scheduler/worker/execute/`：Hub 侧执行规划接入。
- `crates/bastion/src/agent_client/tasks/`：Agent 侧执行规划接入。

## 新增目标驱动

1. **定义驱动标识与能力**
   - 在 `crates/bastion-driver-registry/src/builtins.rs`（或你的 registry 模块）中新增
     `DriverId`（kind + version）与能力位。
2. **实现生命周期行为**
   - 实现 `TargetDriver::store_run`、`cleanup_run`、`snapshot_redacted`。
3. **在驱动内实现读取契约**
   - 实现 `TargetDriver::open_reader`，返回 `TargetRunReader`。
   - Reader 需要覆盖：
     - `complete_exists`
     - `read_bytes`
     - `head_size`
     - `get_to_file`
     - 可选 `local_run_dir`（用于节点本地快速路径）。
4. **保证快照脱敏**
   - `snapshot_redacted` 输出必须不包含原始凭据。
   - 持久化结构保持 `{ node_id, target }`。
5. **确保与规划器兼容**
   - 正确设置能力位，确保 planner 能安全选择 direct/rolling/staged。
6. **补测试**
   - 在 `bastion-driver-registry` 中新增/扩展 `driver_contract_*` 测试。
   - 扩展 planner matrix 测试，覆盖受支持的 source-target-format 组合。

## 共享运行时目标配置映射

请优先使用 `crates/bastion-driver-registry/src/target_runtime.rs` 中的共享 helper，避免在
各模块重复手写 target 映射：

- `runtime_input_for_job_target(...)`
- `runtime_input_for_resolved_target(...)`
- `snapshot_input_for_job_target(...)`
- `driver_id_for_job_target(...)`

这样在新增目标时，Hub 上传、Agent 上传、planner 映射、快照生成、恢复读取、产物流读取
都能保持一致行为。

## 恢复与产物流读取接入

- 恢复与 run-entry 索引 API 通过 `TargetRunReader` 契约读取产物。
- Hub 产物流与恢复共用同一 reader 契约；仅在 reader 提供 `local_run_dir` 且运行位于远端
  agent 时保留本地快速路径分支。
- 对非本地 reader，大文件索引/分片读取统一走 `head_size + get_to_file` 语义。

## 新增来源驱动

1. 在 Hub + Agent 的 planner 适配层增加来源标识/版本映射。
2. 定义来源能力位和 planner 所需策略输入。
3. 在两个运行时（`bastion-engine` 与 `bastion` agent）实现构建链路接入。
4. 在 run events 中输出 planner 字段，并在 run summary 包含 `planner`。
5. 为新来源补齐与所有支持目标/格式的 planner matrix 测试。

## 可观测性要求

- Planner 事件字段至少包含：
  - `source_driver`
  - `target_driver`
  - `plan_mode`
  - `plan_fallback_reason`（发生回退时）
- 运行摘要在成功路径与策略失败路径都应携带 `planner`。

## CI 质量门禁

`bash scripts/ci.sh` 在全量测试前会执行显式门禁：

- `cargo test -p bastion-driver-registry driver_contract`
- `cargo test -p bastion-core execution_planner_matrix`

CI 工作流也新增了专用 `driver-contract-matrix` 任务执行同样检查。
