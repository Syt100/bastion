# 通知（企业微信机器人 + 邮件）

Bastion 支持在每次运行结束时发送通知（成功/失败/被拒绝）。

在 Web UI：

- **设置 → 通知**

## 总览

通知控制分三层：

1. **全局开关**（开/关）
2. **渠道开关**（企业微信 / 邮件）
3. **目的地开关**（启用/禁用某个具体目的地）

任务侧可以选择：

- **继承全局（Inherit）**：发送到所有启用的目的地
- **自定义（Custom）**：只发送到该任务选择的目的地

## 1）开启通知与渠道

在 **通知 → 通知渠道**：

- 打开 **通知**
- 打开你需要的渠道

## 2）创建目的地（凭据）

目的地底层由 Hub 中的加密 secrets 支撑。

在 **通知 → 目的地**：

### 企业微信机器人目的地

创建时需要：

- **Name**：目的地名称（供任务引用）
- **Webhook URL**：企业微信群机器人 webhook URL

### 邮件（SMTP）目的地

创建时需要：

- **Name**：目的地名称（供任务引用）
- **Host / port**
- **TLS mode**：`starttls` / `implicit` / `none`
- **Username / password**（若 SMTP 需要认证）
- **From**
- **To**：一个或多个收件人地址（每行一个或逗号分隔）

你还可以：

- 启用/禁用某个目的地
- **Test**（立即发送一条测试通知）

## 3）自定义 Templates（可选）

在 **通知 → 模板** 可编辑：

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

## 4）按任务配置

在任务编辑器的 **通知** 步骤：

- **继承全局（Inherit）**：使用所有启用的目的地
- **自定义（Custom）**：为该任务选择目的地

被禁用的目的地会在运行结束时被忽略。

## 5）队列与重试

在 **通知 → 队列** 可以查看 queued/sending/sent/failed 项。

动作：

- **立即重试（Retry now）**：对失败项立即重试（前提是全局/渠道/目的地都已启用）
- **取消（Cancel）**：取消仍在 queued 的项

提示：当通知（全局/渠道/目的地）被禁用时，Bastion 会自动取消队列中的相关项目。
