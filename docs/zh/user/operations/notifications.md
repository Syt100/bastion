# 通知（WeCom bot + email）

Bastion 支持在每次 run 结束时发送通知（success/failed/rejected）。

在 Web UI：

- **Settings → Notifications**

## 总览

通知控制分三层：

1. **全局开关**（开/关）
2. **Channel 开关**（WeCom bot / email）
3. **Destination 开关**（启用/禁用某个具体目的地）

Job 侧可以选择：

- **Inherit**：发送到所有启用的 destinations
- **Custom**：只发送到该 job 选择的 destinations

## 1）开启通知与 Channels

在 **Notifications → Channels**：

- 打开 **Notifications**
- 打开你需要的 channels

## 2）创建 Destinations（凭据）

Destinations 底层由 Hub 中的加密 secrets 支撑。

在 **Notifications → Destinations**：

### WeCom bot destination

创建时需要：

- **Name**：destination 名称（供 jobs 引用）
- **Webhook URL**：WeCom bot webhook URL

### Email（SMTP）destination

创建时需要：

- **Name**：destination 名称（供 jobs 引用）
- **Host / port**
- **TLS mode**：`starttls` / `implicit` / `none`
- **Username / password**（若 SMTP 需要认证）
- **From**
- **To**：一个或多个收件人地址

你还可以：

- 启用/禁用某个 destination
- **Test**（立即发送一条测试通知）

## 3）自定义 Templates（可选）

在 **Notifications → Templates** 可编辑：

- WeCom Markdown 模板
- Email subject 模板
- Email body 模板（纯文本）

模板是“占位符替换”（不是完整的模板语言）。

### 可用占位符

- `{{title}}`
- `{{job_id}}`、`{{job_name}}`
- `{{run_id}}`
- `{{status}}`、`{{status_text}}`
- `{{started_at}}`、`{{ended_at}}`
- `{{target_type}}`、`{{target_location}}`、`{{target}}`
- `{{error}}`
- `{{target_line_wecom}}`、`{{error_line_wecom}}`
- `{{target_line_email}}`、`{{error_line_email}}`

## 4）按 Job 配置

在 job 编辑器的 **Notifications** 步骤：

- **Inherit**：使用所有启用的 destinations
- **Custom**：为该 job 选择 destinations

被禁用的 destinations 会在 run 结束时被忽略。

## 5）队列与重试

在 **Notifications → Queue** 可以查看 queued/sending/sent/failed 项。

动作：

- **Retry now**：对失败项立即重试（前提是 global/channel/destination 都已启用）
- **Cancel**：取消仍在 queued 的项

提示：当通知（全局/频道/目的地）被禁用时，Bastion 会自动取消队列中的相关项目。

