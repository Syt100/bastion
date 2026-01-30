# 升级与回滚

本文面向小规模部署（个人/家庭/小团队），重点是：**升级尽量不丢数据**。

## 核心原则：把升级当作“可能不可逆”

Bastion 使用 SQLite，并会在新版本启动时执行数据库迁移（migration）。

因此，**不保证** 你安装旧版本后还能直接读取“已迁移过”的数据库。

想要可靠回滚，通常需要恢复升级前备份的数据目录。

## 升级前准备

1. 确认 **数据目录** 位置（SQLite + `master.key`）：
   - 见：[数据目录](/zh/user/operations/data-directory)
2. 停止写入：
   - 停止 Hub 进程/服务
   - 不要让多个 Bastion 版本同时读写同一个数据目录
3. 备份数据目录（建议至少备份）：
   - `bastion.db`
   - `master.key`
4. （可选但推荐）导出带密码保护的 keypack：

```bash
bastion keypack export --out /secure/location/bastion-keypack.json --password-stdin
```

## 升级后验证

升级完成后建议验证：

- 服务是否在运行（systemd / Windows 服务 / 容器状态）
- Web UI 是否可打开
- （如使用客户端）客户端是否能重新连接
- 健康检查接口：
  - `GET /api/health`（存活）
  - `GET /api/ready`（就绪）
- 在宿主机上运行 `bastion doctor`（如果可用），排查常见部署问题

## 按安装方式升级

### Docker（多数用户推荐）

前提：
- 数据目录通过 Docker volume 或 bind mount 持久化

步骤：

1. 停止容器：
   - `docker compose down`（或 `docker stop ...`）
2. 备份 volume/bind mount（复制到安全位置）。
3. 更新镜像 tag（建议固定到具体版本），然后启动：
   - `docker compose pull && docker compose up -d`
4. 验证（见上文）。

回滚：

1. 停止容器。
2. 恢复升级前备份的 volume/bind mount。
3. 使用旧的镜像 tag 启动容器。

### Linux 包（.deb/.rpm + systemd）

步骤：

1. 停止服务：

```bash
sudo systemctl stop bastion
```

2. 备份数据目录：
   - 包安装方式默认使用 `/var/lib/bastion`（通过 `/etc/bastion/bastion.env`）
3. 安装新包：
   - Debian/Ubuntu：`sudo dpkg -i bastion-<version>-x86_64-unknown-linux-gnu.deb`
   - Fedora/RHEL/openSUSE：`sudo rpm -Uvh bastion-<version>-x86_64-unknown-linux-gnu.rpm`
4. 重新加载 systemd unit（即使没有变化也安全）：

```bash
sudo systemctl daemon-reload
```

5. 启动服务（安装包不会自动启动）：

```bash
sudo systemctl start bastion
```

回滚：

1. 停止服务。
2. 安装旧版本包。
3. 恢复升级前备份的数据目录。
4. 再启动服务。

### Windows MSI（服务安装）

步骤：

1. 停止服务：
   - 打开服务管理器（`services.msc`）-> `Bastion` -> 停止
   - 或：`sc stop Bastion`
2. 备份数据目录：
   - 常见默认：`%PROGRAMDATA%\\bastion\\data`
3. 安装新 MSI。
4. 启动服务：
   - `services.msc` -> 启动
   - 或：`sc start Bastion`

回滚：

1. 停止服务。
2. 安装旧版本 MSI（或卸载后再安装旧版本）。
3. 恢复升级前备份的数据目录。
4. 再启动服务。

### 便携版 tar/zip

推荐做法：

- 用 `BASTION_DATA_DIR` 把数据目录固定到一个稳定路径。
- 每个版本放到单独的目录（回滚时“切回旧二进制”即可）。

步骤：

1. 停止进程。
2. 备份数据目录。
3. 用新版本替换二进制并启动。

回滚：

1. 停止进程。
2. 恢复升级前备份的数据目录。
3. 启动旧版本二进制。
