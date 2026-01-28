# 快速开始

本指南假设是小规模部署（一个 Hub，可选 Agents）。

## 1. 安装

当前官方发布的制品格式：

- **Linux**：
  - `*.tar.gz`（便携版，包含 `bastion` 二进制）
  - `*.deb`（Debian/Ubuntu）
  - `*.rpm`（Fedora/RHEL/openSUSE）
- **Windows**：
  - `*.zip`（便携版，包含 `bastion.exe`）
  - `*.msi`（安装包）
- **macOS**：
  - `*.tar.gz`（便携版，包含 `bastion` 二进制；x64 + arm64）

示例：

- Linux `.tar.gz` / macOS `.tar.gz`：
  - `tar -xzf bastion-<version>-<platform>.tar.gz`
  - `./bastion`
- Debian/Ubuntu `.deb`：
  - `sudo dpkg -i bastion-<version>-linux-x64.deb`
  - 使用 `bastion` 启动
- Fedora/RHEL/openSUSE `.rpm`：
  - `sudo rpm -Uvh bastion-<version>-linux-x64.rpm`
  - 使用 `bastion` 启动
- Windows `.msi`：
  - 运行 MSI 安装
  - 通过 `C:\\Program Files\\Bastion\\bastion.exe` 启动（MSI 默认不会写入 PATH）

你也可以从源码构建（见 [开发文档](/zh/dev/)）。

## 2. 启动 Hub

在负责“统一编排备份”的那台机器上运行 Hub：

```bash
./bastion
```

默认行为：

- 监听 `127.0.0.1:9876`
- 将状态存储在 **数据目录** 中（SQLite + 加密 secrets）

常用参数：

- `--host <ip>` / `BASTION_HOST=<ip>`
- `--port <port>` / `BASTION_PORT=<port>`
- `--data-dir <path>` / `BASTION_DATA_DIR=<path>`

> 在局域网/开发环境（不启用 TLS）下，可以用 `--insecure-http` / `BASTION_INSECURE_HTTP=1` 绑定到非 loopback 地址。
> 对外提供服务时，建议放到终止 TLS 的反向代理后面（见 [反向代理](/zh/user/operations/reverse-proxy)）。

## 3. 首次初始化（创建第一个用户）

打开 Web UI：

- `http://127.0.0.1:9876/`

第一次启动时需要初始化（创建第一个用户）。之后按正常流程登录即可。

注：当前 MVP 暂时只提供单用户初始化流程（暂无用户管理 UI）。

## 4. 下一步

- （可选）[接入 Agents](/zh/user/agents)，在其他机器上执行备份。
- 创建 [Jobs（作业）](/zh/user/jobs)。
- 关注 [Runs（运行记录）](/zh/user/runs)，并通过 [恢复与校验](/zh/user/restore-verify) 做恢复演练。
- 如果需要远端存储，配置 [存储（WebDAV）](/zh/user/storage)。
- 在 [备份快照](/zh/user/backup-snapshots) 中管理备份产物（固定/删除/保留策略）。
- （可选）配置 [通知](/zh/user/operations/notifications)。
