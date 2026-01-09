mod events;
mod ingest;
mod request;

use std::path::Path;

use url::Url;

use super::storage::{OfflineRunEventV1, OfflineRunFileV1, OfflineRunStatusV1, offline_runs_dir};

pub(super) async fn sync_offline_runs(
    base_url: &Url,
    agent_key: &str,
    data_dir: &Path,
) -> Result<(), anyhow::Error> {
    let root = offline_runs_dir(data_dir);
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    let mut run_dirs = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            run_dirs.push(entry.path());
        }
    }
    run_dirs.sort();

    let ingest_url = base_url.join("agent/runs/ingest")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for dir in run_dirs {
        let run_path = dir.join("run.json");
        let bytes = tokio::fs::read(&run_path).await?;
        let run: OfflineRunFileV1 = serde_json::from_slice(&bytes)?;

        if run.status == OfflineRunStatusV1::Running {
            continue;
        }
        let ended_at = run.ended_at.unwrap_or(run.started_at);
        let status = match run.status {
            OfflineRunStatusV1::Success => "success",
            OfflineRunStatusV1::Failed => "failed",
            OfflineRunStatusV1::Rejected => "rejected",
            OfflineRunStatusV1::Running => continue,
        };

        let events_path = dir.join("events.jsonl");
        let events = events::load_offline_events(&events_path).await?;

        let req = request::AgentIngestRunRequestV1::from_offline_run(run, ended_at, status, events);
        ingest::post_offline_run(&client, &ingest_url, agent_key, &req).await?;

        tokio::fs::remove_dir_all(&dir).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
