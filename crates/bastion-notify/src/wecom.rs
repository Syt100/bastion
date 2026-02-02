use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WecomWebhookResponse {
    errcode: i64,
    errmsg: String,
}

fn validate_wecom_webhook_response(
    status: reqwest::StatusCode,
    body: &str,
) -> Result<(), anyhow::Error> {
    if !status.is_success() {
        anyhow::bail!("wecom webhook http {status}: {body}");
    }

    let parsed: WecomWebhookResponse = serde_json::from_str(body)?;
    if parsed.errcode != 0 {
        anyhow::bail!("wecom webhook error {}: {}", parsed.errcode, parsed.errmsg);
    }

    Ok(())
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
    validate_wecom_webhook_response(status, &body)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_wecom_webhook_response;

    #[test]
    fn validate_ok_response_passes() -> Result<(), anyhow::Error> {
        validate_wecom_webhook_response(reqwest::StatusCode::OK, r#"{"errcode":0,"errmsg":"ok"}"#)
    }

    #[test]
    fn validate_non_success_http_status_fails() {
        let err = validate_wecom_webhook_response(reqwest::StatusCode::BAD_GATEWAY, "oops")
            .expect_err("expected error");
        assert!(err.to_string().contains("wecom webhook http"));
    }

    #[test]
    fn validate_errcode_nonzero_fails() {
        let err = validate_wecom_webhook_response(
            reqwest::StatusCode::OK,
            r#"{"errcode":40058,"errmsg":"bad token"}"#,
        )
        .expect_err("expected error");
        assert!(err.to_string().contains("wecom webhook error 40058"));
    }

    #[test]
    fn validate_invalid_json_fails() {
        assert!(validate_wecom_webhook_response(reqwest::StatusCode::OK, "not json").is_err());
    }
}
