use std::path::Path;

use tokio::io::AsyncBufReadExt as _;

use super::OfflineRunEventV1;

pub(super) async fn load_offline_events(
    events_path: &Path,
) -> Result<Vec<OfflineRunEventV1>, anyhow::Error> {
    let mut events = Vec::new();
    match tokio::fs::File::open(events_path).await {
        Ok(file) => {
            let mut lines = tokio::io::BufReader::new(file).lines();
            while let Some(line) = lines.next_line().await? {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let ev: OfflineRunEventV1 = serde_json::from_str(line)?;
                events.push(ev);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(events)
}
