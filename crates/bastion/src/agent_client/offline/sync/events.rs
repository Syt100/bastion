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

#[cfg(test)]
mod tests {
    use super::load_offline_events;

    use super::super::OfflineRunEventV1;

    #[tokio::test]
    async fn load_offline_events_returns_empty_when_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("events.jsonl");

        let events = load_offline_events(&missing).await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn load_offline_events_parses_jsonl_and_skips_blank_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("events.jsonl");

        let ev1 = OfflineRunEventV1 {
            seq: 1,
            ts: 10,
            level: "info".to_string(),
            kind: "start".to_string(),
            message: "a".to_string(),
            fields: None,
        };
        let ev2 = OfflineRunEventV1 {
            seq: 2,
            ts: 11,
            level: "warn".to_string(),
            kind: "step".to_string(),
            message: "b".to_string(),
            fields: Some(serde_json::json!({"x": 1})),
        };

        let content = format!(
            "\n {}\n\n{}\n  \n{}\n",
            serde_json::to_string(&ev1).unwrap(),
            serde_json::to_string(&ev2).unwrap(),
            serde_json::to_string(&ev1).unwrap(),
        );
        tokio::fs::write(&path, content).await.unwrap();

        let events = load_offline_events(&path).await.unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);
        assert_eq!(events[2].seq, 1);
        assert_eq!(events[1].fields, Some(serde_json::json!({"x": 1})));
    }
}
