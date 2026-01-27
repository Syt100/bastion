# Notifications (WeCom bot + email)

Bastion can send per-run notifications when a run finishes (success/failed/rejected).

In the Web UI:

- **Settings → Notifications**

## Overview

Notifications are controlled at three layers:

1. **Global switch** (on/off)
2. **Channel switch** (WeCom bot / email)
3. **Destination switch** (enable/disable individual destinations)

Jobs can either:

- **Inherit**: send to all enabled destinations, or
- **Custom**: send only to selected destinations for that job

## 1) Enable notifications and channels

In **Notifications → Channels**:

- Enable **Notifications**
- Enable the channels you want to use

## 2) Create destinations (credentials)

Destinations are backed by encrypted secrets stored on the Hub.

In **Notifications → Destinations**:

### WeCom bot destination

Create a destination with:

- **Name**: destination name (used by jobs)
- **Webhook URL**: the WeCom bot webhook URL

### Email (SMTP) destination

Create a destination with:

- **Name**: destination name (used by jobs)
- **Host / port**
- **TLS mode**: `starttls` / `implicit` / `none`
- **Username / password** (if required by your SMTP server)
- **From**
- **To**: one or more recipient addresses

You can also:

- **Enable/disable** a destination
- **Test** a destination (sends an immediate test message)

## 3) Customize templates (optional)

In **Notifications → Templates** you can edit:

- WeCom Markdown template
- Email subject template
- Email body template (plain text)

Templates are simple placeholder replacement (not a full template language).

### Available placeholders

- `{{title}}`
- `{{job_id}}`, `{{job_name}}`
- `{{run_id}}`
- `{{status}}`, `{{status_text}}`
- `{{started_at}}`, `{{ended_at}}`
- `{{target_type}}`, `{{target_location}}`, `{{target}}`
- `{{error}}`
- `{{target_line_wecom}}`, `{{error_line_wecom}}`
- `{{target_line_email}}`, `{{error_line_email}}`

## 4) Per-job configuration

In the job editor (**Notifications** step):

- **Inherit**: uses all enabled destinations
- **Custom**: pick destination names for each channel

Disabled destinations are ignored when a run finishes.

## 5) Queue and retries

In **Notifications → Queue** you can see queued/sending/sent/failed items.

Actions:

- **Retry now**: re-queues a failed notification (only works if global/channel/destination are enabled)
- **Cancel**: cancels a queued notification

Tip: if notifications are disabled (globally, by channel, or by destination), Bastion cancels queued items automatically.

