use url::Url;

use super::request::AgentIngestRunRequestV1;

pub(super) async fn post_offline_run(
    client: &reqwest::Client,
    ingest_url: &Url,
    agent_key: &str,
    req: &AgentIngestRunRequestV1,
) -> Result<(), anyhow::Error> {
    let res = client
        .post(ingest_url.clone())
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {agent_key}"),
        )
        .json(req)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::NO_CONTENT {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("ingest failed: HTTP {status}: {text}");
    }

    Ok(())
}
