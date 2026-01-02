use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WecomWebhookResponse {
    errcode: i64,
    errmsg: String,
}

pub async fn send_markdown(webhook_url: &str, content: &str) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let res = client
        .post(webhook_url)
        .json(&serde_json::json!({
            "msgtype": "markdown",
            "markdown": { "content": content },
        }))
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;
    if !status.is_success() {
        anyhow::bail!("wecom webhook http {status}: {body}");
    }

    let parsed: WecomWebhookResponse = serde_json::from_str(&body)?;
    if parsed.errcode != 0 {
        anyhow::bail!("wecom webhook error {}: {}", parsed.errcode, parsed.errmsg);
    }

    Ok(())
}
