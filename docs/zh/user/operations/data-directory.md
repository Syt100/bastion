# 数据目录结构与密钥管理

Bastion 会把运行状态（SQLite）与加密凭据存放在一个可配置的数据目录中。

## 数据目录在哪里？

优先级顺序：

1. `--data-dir <path>`（CLI）
2. `BASTION_DATA_DIR=<path>`（环境变量）
3. 如果 `<exe_dir>/data` 可写，则使用该目录
4. 操作系统默认目录：
   - Windows：`%PROGRAMDATA%\\bastion\\data`（如果可用）
   - 否则：由 `directories` crate 决定的应用本地数据目录

说明：

- 对于 Linux `.deb/.rpm` + systemd 的安装方式，安装包会提供 `/etc/bastion/bastion.env`，并设置：
  - `BASTION_DATA_DIR=/var/lib/bastion`
- 对于 Windows MSI（以服务方式运行），默认通常会解析到：
  - `%PROGRAMDATA%\\bastion\\data`

## 目录里有什么？

常见文件/目录：

- `bastion.db`：SQLite 数据库（备份任务、运行记录、配置、加密凭据等）
- `master.key`：本地主密钥（用于加密 `bastion.db` 中的凭据）
- `runs/`：运行期间的临时目录（staging，用于构建/上传产物）
  - 进程被中断时，可能会留下不完整的临时数据。

客户端的数据目录还会包含：

- `agent.json`：接入身份信息（agent_id / agent_key）

## 备份数据目录

至少你应该备份：

- `master.key`
- `bastion.db`

如果丢失了 `master.key`，你将无法解密数据库中已有的加密凭据（例如 WebDAV、SMTP、企业微信 webhook，以及备份加密相关密钥等）。

## Keypack 导出/导入（推荐）

Bastion 提供一种带密码加密的 “keypack” 用于备份 `master.key`。

导出 keypack：

```bash
bastion keypack export --out /secure/location/bastion-keypack.json --password-stdin
```

导入 keypack：

```bash
bastion keypack import --in /secure/location/bastion-keypack.json --password-stdin
```

强制覆盖导入（危险）：

```bash
bastion keypack import --in /secure/location/bastion-keypack.json --password-stdin --force
```

导入或轮换 `master.key` 后，建议重启服务以确保新密钥被加载。

## 主密钥轮换

轮换 `master.key` 的当前主密钥（active key）：

```bash
bastion keypack rotate
```

轮换会保留旧密钥，因此旧凭据仍可被解密；新写入的凭据会使用新的密钥。
