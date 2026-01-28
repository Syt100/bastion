# 数据目录结构与密钥管理

Bastion 会把状态与 secrets 存放在一个可配置的数据目录中。

## 数据目录在哪里？

优先级顺序：

1. `--data-dir <path>`（CLI）
2. `BASTION_DATA_DIR=<path>`（环境变量）
3. 如果 `<exe_dir>/data` 可写，则使用该目录
4. OS fallback 目录：
   - Windows：`%PROGRAMDATA%\\bastion\\data`（如果可用）
   - 否则：由 `directories` crate 决定的 OS 应用数据目录（local）

说明：

- 对于 Linux `.deb/.rpm` + systemd 的安装方式，安装包会提供 `/etc/bastion/bastion.env`，并设置：
  - `BASTION_DATA_DIR=/var/lib/bastion`
- 对于 Windows MSI（以服务方式运行），默认通常会解析到：
  - `%PROGRAMDATA%\\bastion\\data`

## 目录里有什么？

常见文件/目录：

- `bastion.db`：SQLite 数据库（jobs、runs、secrets 元数据等）
- `master.key`：本地 master keyring（用于加密 `bastion.db` 中的 secrets）
- `runs/`：运行期间的临时 staging 目录（用于构建/上传产物）
  - 如果进程被中断，可能会留下不完整的 staging 数据。

Agent 模式还会包含：

- `agent.json`：agent 接入身份（agent_id / agent_key）

## 备份数据目录

至少你应该备份：

- `master.key`
- `bastion.db`

如果丢失了 `master.key`，你将无法解密数据库中已有的加密 secrets（WebDAV 凭据、SMTP、WeCom webhook、备份加密身份等）。

## Keypack 导出/导入（推荐）

Bastion 提供一种带密码加密的 “keypack” 用于备份 `master.key`。

导出 keypack：

```bash
./bastion keypack export --out /secure/location/bastion-keypack.json --password-stdin
```

导入 keypack：

```bash
./bastion keypack import --in /secure/location/bastion-keypack.json --password-stdin
```

强制覆盖导入（危险）：

```bash
./bastion keypack import --in /secure/location/bastion-keypack.json --password-stdin --force
```

导入或 rotate `master.key` 后，建议重启服务以确保新 keyring 被加载。

## Master key 轮转

轮转 `master.key` 的 active key：

```bash
./bastion keypack rotate
```

轮转会保留旧 keys，因此旧 secrets 仍可被解密；新 secrets 会使用新的 active key。
