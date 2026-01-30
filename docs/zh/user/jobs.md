# 备份任务（Jobs）

**备份任务（Job）** 用来定义：备份什么、存到哪里、何时运行。

## 运行位置（Hub vs 客户端）

每个任务只会在一个节点上运行：

- **Hub（本机）**：在 Hub 所在机器上执行。
- **客户端（Agent）**：在某个已接入的客户端机器上执行。

在 Web UI 中，Jobs 页面按 **节点上下文** 展示（`hub` 或某个客户端）。你可以通过主布局中的节点选择器切换查看/创建对应节点的任务。

## 创建与编辑 Job

在 Web UI：

- **Jobs** → **Create**

任务编辑器按步骤组织：

- **基本信息（Basics）**：名称、节点、计划、时区、重叠策略、保留策略
- **数据源（Source）**：备份源（取决于任务类型）
- **备份目标（Target）**：备份存储位置（本地目录 / WebDAV）
- **安全（Security）**：产物格式 + 可选加密
- **通知（Notifications）**：继承或自定义通知目的地
- **预览（Review）**：最终汇总（可选 JSON 预览）

## 调度与重叠

每个任务包含：

- **Schedule mode**
  - **Manual**：不自动触发；通过 **Run now** 手动运行
  - **Simple**：面向普通用户的预设（底层存为 cron）
  - **Cron**：直接填写 cron 表达式
- **Schedule timezone**：解释 schedule 的 IANA 时区（与 OS 时区独立）
  - 新建 job 默认使用 Hub timezone。
- **Overlap policy**
  - **queue**：如果已有运行中的 run，则把触发排队
  - **reject**：如果已有运行中的 run，则拒绝触发（run 状态为 rejected）

## Job 类型（Source 配置）

### Filesystem

备份所选 node 上的文件/目录。

Source 配置：

- **Source paths**：要包含的路径列表
  - **Browse** 按钮基于该 node 的文件系统；如果选择了 agent，则 agent 必须在线才能浏览。
- **Pre-scan**：打包前预估（用于进度/ETA）
- **Include/Exclude**：按行的匹配模式
- **Symlink policy**：keep / follow / skip
- **Hardlink policy**：copy / keep
- **Error policy**：fail fast / skip fail / skip ok

### SQLite

创建在线 SQLite 快照（`sqlite backup` API），并将其作为备份产物打包。

Source 配置：

- **SQLite path**：数据库文件路径（在所选 node 上）
- **Integrity check（可选）**：对快照执行 `PRAGMA integrity_check`，如发现问题则让该 run 失败

### Vaultwarden

备份 Vaultwarden 部署（SQLite `db.sqlite3` + 必要的数据目录内容）。

Source 配置：

- **Vaultwarden data dir**：Vaultwarden 的 `data/` 在宿主机上的挂载路径（在所选 node 上）

具体示例见配方：[Vaultwarden](/zh/user/recipes/vaultwarden)。

## Targets（备份存储位置）

### 本地目录（Local directory）

将备份输出保存到所选 node 的某个目录下：

- **Base dir**：例如 `/opt/bastion-backups`

### WebDAV

将备份输出上传到 WebDAV：

- **Base URL**：例如 `https://dav.example.com/backups`
- **Secret name**：选择在 Bastion 中保存的凭据名（WebDAV secrets 是 **node-scoped**）

在 **Settings → Storage** 中管理凭据，并在需要时分发到 agents：[存储（WebDAV）](/zh/user/storage)。

### Part size

Target 支持设置 **part size**（MiB）。较大的备份会被拆分为多个 part，以避免单文件过大，并使重试成本更低。

## 产物格式与加密

### Format

- **archive_v1**：压缩归档格式（推荐默认）
- **raw_tree_v1**：原始文件树格式（不支持 payload 加密）

注：Vaultwarden job 当前仅支持 **archive_v1**。

### Encryption（age）

对 `archive_v1` 可以启用 payload 加密（age x25519）。

- **Encryption key name** 是一个标签（默认：`default`）
- Hub 在首次使用时会自动创建该 key
- Agent 只会拿到用于加密的 public recipient；当需要把加密备份 restore 到 agent 时，Hub 会在派发 restore 前自动确保 agent 具备所需私钥

## 快照与保留策略

成功的 run 会生成一个 **快照（snapshot）**。你可以：

- 在任务维度查看/固定/删除快照：[备份快照](/zh/user/backup-snapshots)
- 在任务上配置保留策略：
  - 保留最近 N 份 / 保留最近 N 天内的快照
  - 安全阀：max delete per tick / per day
  - 新建任务的 **默认值**来自 **Settings → Runtime config**

## 通知（Notifications）

Job 支持按 run 发送通知（WeCom bot + email）。

- **Inherit**：发送到所有启用的 destinations
- **Custom**：为该 job 指定 destinations（被禁用的 destination 会被忽略）

在 **Settings → Notifications** 中配置 channels/destinations/templates。

另见：[通知](/zh/user/operations/notifications)。

## 批量部署（Clone job to nodes）

UI 提供批量 **Deploy to nodes** 功能，用于把一个 job 克隆到多个 agents。

Deploy 会做的事：

- 为每个选中的客户端创建一个新的任务
- 继承源任务的配置 + 计划 + 时区 + 重叠策略
- 按节点校验（例如：缺少该节点的 WebDAV 凭据）
- 创建后触发一次配置同步（离线客户端会在下次连接后应用）

命名模板：

- 默认：`{name} ({node})`
- 占位符：`{name}`、`{node}`
- 冲突会自动加后缀（`#2`、`#3`…）

进度在 **设置 → 批量操作** 中查看。

## 运行记录、恢复与校验

从一个任务你可以打开：

- **运行记录（Runs）**：历史记录（状态、时间、错误）
- **快照（Snapshots）**：备份产物

成功运行支持：

- **恢复（Restore）**：将备份恢复到本地目录或 WebDAV（支持全量或选择性恢复）
- **校验（Verify）**：恢复到临时目录并做哈希校验；适用时也会做 SQLite 完整性检查

另见：

- [Runs（运行记录）](/zh/user/runs)
- [恢复与校验](/zh/user/restore-verify)

## 归档与删除 Job

任务可以归档（停止调度并从默认视图隐藏）。

- **Archive**：停止调度并隐藏任务；可选“同时删除快照”（会跳过已固定的快照）
- **Unarchive**：恢复为活跃状态
- **Delete**：从 Hub 数据库中永久删除任务与运行历史（与快照删除是不同概念）
