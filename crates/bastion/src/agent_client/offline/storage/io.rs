use std::path::Path;

use tokio::io::AsyncWriteExt as _;

use super::types::{OfflineRunEventV1, OfflineRunFileV1};

pub(super) async fn write_offline_run_file_atomic(
    path: &Path,
    doc: OfflineRunFileV1,
) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(&doc)?;
    let tmp = path.with_extension("json.partial");
    let _ = tokio::fs::remove_file(&tmp).await;
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

pub(super) async fn append_offline_event_line(
    path: &Path,
    event: &OfflineRunEventV1,
) -> Result<(), anyhow::Error> {
    let line = serde_json::to_vec(event)?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(&line).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{OfflineRunEventV1, OfflineRunFileV1, OfflineRunStatusV1};
    use super::{append_offline_event_line, write_offline_run_file_atomic};

    #[tokio::test]
    async fn write_offline_run_file_atomic_writes_file_and_cleans_up_tmp() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("run.json");

        let doc = OfflineRunFileV1 {
            v: 1,
            id: "run1".to_string(),
            job_id: "job1".to_string(),
            job_name: "job name".to_string(),
            status: OfflineRunStatusV1::Running,
            started_at: 10,
            ended_at: None,
            summary: Some(serde_json::json!({"k": "v"})),
            error: None,
        };

        write_offline_run_file_atomic(&path, doc).await.unwrap();

        let raw = std::fs::read(&path).unwrap();
        let parsed: OfflineRunFileV1 = serde_json::from_slice(&raw).unwrap();
        assert_eq!(parsed.v, 1);
        assert_eq!(parsed.id, "run1");
        assert_eq!(parsed.job_id, "job1");
        assert_eq!(parsed.status, OfflineRunStatusV1::Running);
        assert_eq!(parsed.summary, Some(serde_json::json!({"k": "v"})));

        assert!(!path.with_extension("json.partial").exists());
    }

    #[tokio::test]
    async fn append_offline_event_line_appends_jsonl_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("events.jsonl");

        append_offline_event_line(
            &path,
            &OfflineRunEventV1 {
                seq: 1,
                ts: 10,
                level: "info".to_string(),
                kind: "start".to_string(),
                message: "a".to_string(),
                fields: None,
            },
        )
        .await
        .unwrap();
        append_offline_event_line(
            &path,
            &OfflineRunEventV1 {
                seq: 2,
                ts: 11,
                level: "warn".to_string(),
                kind: "step".to_string(),
                message: "b".to_string(),
                fields: Some(serde_json::json!({"x": 1})),
            },
        )
        .await
        .unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let lines = text.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);

        let e1: OfflineRunEventV1 = serde_json::from_str(lines[0]).unwrap();
        let e2: OfflineRunEventV1 = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert_eq!(e2.fields, Some(serde_json::json!({"x": 1})));
    }
}
