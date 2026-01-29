# 默认行为与配置优先级

本文汇总 Bastion 的默认行为，以及配置是如何解析生效的。

## Hub 的默认行为

当你不带任何参数运行 `bastion` 时：

- 监听地址：`127.0.0.1:9876`
- 数据目录：自动解析（见下文）
- Web UI：`http://127.0.0.1:9876/`

## 数据目录解析顺序

优先级（从高到低）：

1. `--data-dir <path>`（CLI）
2. `BASTION_DATA_DIR=<path>`（环境变量）
3. 如果 `<exe_dir>/data` 可写，则使用它
4. OS fallback 目录：
   - Windows：`%PROGRAMDATA%\\bastion\\data`（如果可用）
   - 否则：系统应用数据目录（由 `directories` crate 提供）

另见：[数据目录](/zh/user/operations/data-directory)。

## HTTP 安全默认值

默认情况下 Bastion 只监听 loopback（`127.0.0.1`），用于本机访问。

如果你想在局域网访问（不启用 TLS），需要显式开启不安全模式：

- `--insecure-http`
- 或 `BASTION_INSECURE_HTTP=1`

如果要对公网提供服务，请放到终止 TLS 的反向代理后。

另见：[反向代理](/zh/user/operations/reverse-proxy)。

## 配置优先级（CLI / env / 数据库 / 默认值）

不同配置项来源不同：

- **CLI flags**：优先级最高
- **环境变量**：其次
- **数据库**：仅适用于 Web UI “运行时配置”页面管理的字段
- **内置默认值**：最低

对于会在 Web UI 运行时配置页面出现的字段，其优先级为：

1. CLI flags
2. 环境变量
3. 数据库保存值
4. 内置默认值

另见：[运行时配置](/zh/user/operations/runtime-config)。

提示：`bastion config` 会显示各字段的 effective 值以及它来自哪里。
