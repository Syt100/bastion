# 可观测性（健康检查、就绪检查、请求 ID）

对小规模部署来说，最实用的可观测性通常是：

- 存活/就绪探针
- 稳定的请求 ID（把 UI 看到的错误与服务端日志关联起来）
- 清晰的日志位置（控制台 / journald / 文件）

## 健康检查接口

这些接口都会返回一个很小的 JSON，并包含 `ok` 布尔值。

### 存活（Liveness）：`GET /api/health`

回答的问题是：“进程是否还活着？”

- 期望：`200` + `{ "ok": true }`

### 就绪（Readiness）：`GET /api/ready`

回答的问题是：“Hub 是否已准备好对外服务？”

readiness 会检查关键依赖（包括数据库连通性）。

- 期望：`200` + `{ "ok": true }`
- 依赖未就绪：`503` + `{ "ok": false }`

### 系统信息：`GET /api/system`

用于获取一些基础运行信息，例如：

- 版本号（tag 或 crate version）
- 构建时间（如果可用）
- 是否开启 `--insecure-http`
- Hub 时区

## 请求 ID

Bastion 会在 HTTP 响应头中返回 `x-request-id`。

当你反馈问题时，建议附上：

- 请求 URL
- `x-request-id` 的值
- 日志中与该 request-id 相关的片段

这样更容易把客户端错误与服务端日志对应起来。

提示：Web UI 的错误弹窗会展示 request ID，反馈问题时建议一并提供。

## 日志

见：[日志](/zh/user/operations/logging)。

快速提示：

- 默认：输出到控制台
- systemd：控制台日志会被 journald 接管（`journalctl -u bastion -f`）
- Windows 服务：默认写入 `%PROGRAMDATA%\\bastion\\logs\\bastion.log`（除非你显式配置）
