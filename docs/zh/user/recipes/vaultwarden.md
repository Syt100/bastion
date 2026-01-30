# Vaultwarden 备份配方（Docker/Compose，SQLite）

本指南介绍：当 Vaultwarden 以 Docker/Compose 部署并使用 SQLite（很多部署的默认选项）时，如何用 Bastion 进行备份。

## 前置条件

- Vaultwarden 将数据存放在宿主机挂载的 `data/` 目录中。
- Bastion 运行在同一宿主机（Hub 模式），或者你在 Vaultwarden 宿主机上运行一个客户端（Agent）。
- 本配方 **不需要** 停止 Vaultwarden 服务。

## 示例 docker-compose.yml（Vaultwarden）

```yaml
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: unless-stopped
    ports:
      - "8081:80"
    volumes:
      - /opt/vaultwarden/data:/data
```

在该示例中，宿主机路径为 `/opt/vaultwarden/data`。

## 配置备份任务

在 Bastion Web UI：

1. 创建一个新任务，类型选择 **Vaultwarden**
2. 将 **Vaultwarden data dir** 设置为挂载目录对应的宿主机路径：
   - 示例：`/opt/vaultwarden/data`
3. 选择一个备份目标：
   - **Local directory**（最简单）：`/opt/bastion-backups`
   - **WebDAV**：先在 Settings 中配置 WebDAV 凭据
4. （可选）启用 SQLite `PRAGMA integrity_check`
5. （可选）启用备份加密（age）

Bastion 会备份的内容：

- SQLite 数据库：`<data_dir>/db.sqlite3`（通过 SQLite 在线备份 API 生成快照）
- Vaultwarden 数据目录中用于恢复所需的文件（例如 attachments/keys 等相关内容）

## 校验（Verify，推荐）

使用 UI 中的 **校验（Verify）** 做端到端完整性检查：

- 下载快照
- 恢复到临时目录
- 做文件哈希校验
- 对 SQLite 备份执行 `PRAGMA integrity_check`

## 说明

- 当前 Vaultwarden 配方仅支持 SQLite。
- 如果你的 Vaultwarden 使用 MySQL/PostgreSQL，请等待 Bastion 的数据库备份原语支持这些引擎后，再扩展对应配方。
