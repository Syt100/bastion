# 日志（Logging）

## 默认行为

- Bastion 默认将日志输出到控制台。
- 在 systemd 下运行时，控制台日志会被 journald 接管：
  - `journalctl -u bastion -f`
- 在 Windows 服务模式（MSI 安装）下，如果你没有显式配置 `--log-file` / `BASTION_LOG_FILE`，Bastion 默认会写入：
  - `%PROGRAMDATA%\\bastion\\logs\\bastion.log`
- 如果没有显式配置 filter，Bastion 会使用一个相对保守的默认 filter：
  - `info,tower_http=warn`
  - 该配置会保留 Bastion 的 `INFO` 日志，同时抑制较吵的逐请求 HTTP access logs。

## Log Filter（日志级别/范围）

你可以通过以下任一种方式配置 log filter：

- `--log "<filter>"`
- `BASTION_LOG="<filter>"`
- `RUST_LOG="<filter>"`

语法遵循 `tracing_subscriber::EnvFilter` 的标准格式。

你也可以在 Web UI 中设置日志：

- **Settings → Runtime config**

另见：[运行时配置](/zh/user/operations/runtime-config)。

注意：

- 部分改动需要重启 Hub 才会生效。
- CLI/env 的设置会覆盖 UI 保存值（UI 中会标记为 overridden）。

示例：

```bash
# 默认（INFO）
bastion

# 更详细的 Bastion 日志，同时保持 HTTP 访问日志安静
bastion --log "bastion=debug,tower_http=warn"

# 同时开启 HTTP 请求日志
bastion --log "info,tower_http=info"
```

## 文件日志与轮转（Rotation）

开启文件日志（控制台 + 文件同时输出）：

- `--log-file /path/to/bastion.log`
- 或 `BASTION_LOG_FILE=/path/to/bastion.log`

轮转选项：

- `--log-rotation daily|hourly|never`（默认：`daily`）
- `BASTION_LOG_ROTATION=daily|hourly|never`

保留策略（仅对轮转出来的文件生效）：

- `--log-keep-files <N>`（默认：`30`，`0` 表示不清理）
- `BASTION_LOG_KEEP_FILES=<N>`

说明：

- 轮转文件使用配置的文件名作为前缀：
  - daily：`bastion.log.YYYY-MM-DD`
  - hourly：`bastion.log.YYYY-MM-DD-HH`
- Bastion 只会清理符合上述轮转命名模式的文件。

示例：

```bash
bastion \
  --log-file ./data/logs/bastion.log \
  --log-rotation daily \
  --log-keep-files 30
```

## Secret 脱敏（Redaction）

Bastion **禁止**在日志中输出敏感信息（密码、token、私钥等）。

如果你把凭据直接写进 URL，Bastion 会尽力在日志中做脱敏，但仍建议避免在 URL 中嵌入 secrets。
