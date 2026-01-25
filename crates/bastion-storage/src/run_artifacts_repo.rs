use serde::Serialize;
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;

use crate::runs_repo::{self, RunStatus};

#[derive(Debug, Clone, Serialize)]
pub struct RunArtifact {
    pub run_id: String,
    pub job_id: String,
    pub node_id: String,
    pub target_type: String,
    pub target_snapshot: serde_json::Value,
    pub artifact_format: String,
    pub status: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub source_files: Option<u64>,
    pub source_dirs: Option<u64>,
    pub source_bytes: Option<u64>,
    pub transfer_bytes: Option<u64>,
    pub last_error_kind: Option<String>,
    pub last_error: Option<String>,
    pub last_attempt_at: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
struct RunTargetSnapshot {
    node_id: String,
    target: RunTarget,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RunTarget {
    Webdav {
        #[allow(dead_code)]
        base_url: String,
        #[allow(dead_code)]
        secret_name: String,
    },
    LocalDir {
        #[allow(dead_code)]
        base_dir: String,
    },
}

fn parse_row(row: &sqlx::sqlite::SqliteRow) -> Result<RunArtifact, anyhow::Error> {
    let snapshot_json = row.get::<String, _>("target_snapshot_json");
    let target_snapshot = serde_json::from_str::<serde_json::Value>(&snapshot_json)?;

    Ok(RunArtifact {
        run_id: row.get::<String, _>("run_id"),
        job_id: row.get::<String, _>("job_id"),
        node_id: row.get::<String, _>("node_id"),
        target_type: row.get::<String, _>("target_type"),
        target_snapshot,
        artifact_format: row.get::<String, _>("artifact_format"),
        status: row.get::<String, _>("status"),
        started_at: row.get::<i64, _>("started_at"),
        ended_at: row.get::<i64, _>("ended_at"),
        source_files: row
            .get::<Option<i64>, _>("source_files")
            .and_then(|v| u64::try_from(v).ok()),
        source_dirs: row
            .get::<Option<i64>, _>("source_dirs")
            .and_then(|v| u64::try_from(v).ok()),
        source_bytes: row
            .get::<Option<i64>, _>("source_bytes")
            .and_then(|v| u64::try_from(v).ok()),
        transfer_bytes: row
            .get::<Option<i64>, _>("transfer_bytes")
            .and_then(|v| u64::try_from(v).ok()),
        last_error_kind: row.get::<Option<String>, _>("last_error_kind"),
        last_error: row.get::<Option<String>, _>("last_error"),
        last_attempt_at: row.get::<Option<i64>, _>("last_attempt_at"),
    })
}

pub async fn get_run_artifact(
    db: &SqlitePool,
    run_id: &str,
) -> Result<Option<RunArtifact>, anyhow::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          run_id, job_id, node_id, target_type, target_snapshot_json,
          artifact_format, status, started_at, ended_at,
          source_files, source_dirs, source_bytes, transfer_bytes,
          last_error_kind, last_error, last_attempt_at
        FROM run_artifacts
        WHERE run_id = ?
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| parse_row(&r)).transpose()?)
}

pub async fn list_run_artifacts_for_job(
    db: &SqlitePool,
    job_id: &str,
    cursor: u64,
    limit: u64,
    status: Option<&str>,
) -> Result<Vec<RunArtifact>, anyhow::Error> {
    let limit = limit.clamp(1, 200);

    // NOTE: keep query composition simple; add more filters as needed.
    let mut sql = String::from(
        r#"
        SELECT
          run_id, job_id, node_id, target_type, target_snapshot_json,
          artifact_format, status, started_at, ended_at,
          source_files, source_dirs, source_bytes, transfer_bytes,
          last_error_kind, last_error, last_attempt_at
        FROM run_artifacts
        WHERE job_id = ?
        "#,
    );
    if status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY ended_at DESC, run_id DESC LIMIT ? OFFSET ?");

    let mut q = sqlx::query(&sql).bind(job_id);
    if let Some(s) = status {
        q = q.bind(s);
    }
    let rows = q
        .bind(limit as i64)
        .bind(cursor as i64)
        .fetch_all(db)
        .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(parse_row(&row)?);
    }
    Ok(out)
}

pub async fn mark_run_artifact_deleting(
    db: &SqlitePool,
    run_id: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE run_artifacts SET status = 'deleting', updated_at = ?, last_error_kind = NULL, last_error = NULL, last_attempt_at = NULL WHERE run_id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_run_artifact_deleted(
    db: &SqlitePool,
    run_id: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE run_artifacts SET status = 'deleted', updated_at = ?, last_error_kind = NULL, last_error = NULL, last_attempt_at = NULL WHERE run_id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_run_artifact_missing(
    db: &SqlitePool,
    run_id: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE run_artifacts SET status = 'missing', updated_at = ?, last_error_kind = NULL, last_error = NULL, last_attempt_at = NULL WHERE run_id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_run_artifact_error(
    db: &SqlitePool,
    run_id: &str,
    last_error_kind: &str,
    last_error: &str,
    last_attempt_at: i64,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE run_artifacts SET status = 'error', updated_at = ?, last_error_kind = ?, last_error = ?, last_attempt_at = ? WHERE run_id = ?",
    )
    .bind(now)
    .bind(last_error_kind)
    .bind(last_error)
    .bind(last_attempt_at)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

fn extract_metrics_from_progress(
    progress: Option<&serde_json::Value>,
) -> (Option<u64>, Option<u64>, Option<u64>, Option<u64>) {
    let Some(p) = progress else {
        return (None, None, None, None);
    };

    let stage = p.get("stage").and_then(|v| v.as_str()).unwrap_or_default();

    let detail = p.get("detail");
    let backup = detail.and_then(|v| v.get("backup"));

    let source_total = backup.and_then(|b| b.get("source_total")).and_then(|v| {
        serde_json::from_value::<bastion_core::progress::ProgressUnitsV1>(v.clone()).ok()
    });

    let transfer_total_bytes = backup
        .and_then(|b| b.get("transfer_total_bytes"))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            if stage != "upload" {
                return None;
            }
            p.get("total")
                .and_then(|t| t.get("bytes"))
                .and_then(|v| v.as_u64())
        });

    let source_files = source_total.map(|t| t.files);
    let source_dirs = source_total.map(|t| t.dirs);
    let source_bytes = source_total.map(|t| t.bytes);

    (source_files, source_dirs, source_bytes, transfer_total_bytes)
}

pub async fn upsert_run_artifact_from_successful_run(
    db: &SqlitePool,
    run_id: &str,
) -> Result<bool, anyhow::Error> {
    let run = runs_repo::get_run(db, run_id).await?;
    let Some(run) = run else {
        return Ok(false);
    };
    if run.status != RunStatus::Success {
        return Ok(false);
    }
    let ended_at = match run.ended_at {
        Some(v) => v,
        None => return Ok(false),
    };

    let snapshot = runs_repo::get_run_target_snapshot(db, run_id).await?;
    let Some(snapshot) = snapshot else {
        return Ok(false);
    };
    let parsed = serde_json::from_value::<RunTargetSnapshot>(snapshot.clone())?;
    let target_type = match parsed.target {
        RunTarget::Webdav { .. } => "webdav",
        RunTarget::LocalDir { .. } => "local_dir",
    };

    let artifact_format = run
        .summary
        .as_ref()
        .and_then(|v| v.get("artifact_format"))
        .and_then(|v| v.as_str())
        .unwrap_or("archive_v1");

    let (source_files, source_dirs, source_bytes, transfer_bytes) =
        extract_metrics_from_progress(run.progress.as_ref());

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let snapshot_json = serde_json::to_string(&snapshot)?;

    sqlx::query(
        r#"
        INSERT INTO run_artifacts (
          run_id, job_id, node_id, target_type, target_snapshot_json,
          artifact_format, status, started_at, ended_at,
          source_files, source_dirs, source_bytes, transfer_bytes,
          created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, 'present', ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(run_id) DO UPDATE SET
          job_id = excluded.job_id,
          node_id = excluded.node_id,
          target_type = excluded.target_type,
          target_snapshot_json = excluded.target_snapshot_json,
          artifact_format = excluded.artifact_format,
          status = excluded.status,
          started_at = excluded.started_at,
          ended_at = excluded.ended_at,
          source_files = excluded.source_files,
          source_dirs = excluded.source_dirs,
          source_bytes = excluded.source_bytes,
          transfer_bytes = excluded.transfer_bytes,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(&run.id)
    .bind(&run.job_id)
    .bind(&parsed.node_id)
    .bind(target_type)
    .bind(snapshot_json)
    .bind(artifact_format)
    .bind(run.started_at)
    .bind(ended_at)
    .bind(source_files.map(|v| v as i64))
    .bind(source_dirs.map(|v| v as i64))
    .bind(source_bytes.map(|v| v as i64))
    .bind(transfer_bytes.map(|v| v as i64))
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};

    use crate::db;
    use crate::jobs_repo::{self, OverlapPolicy};
    use crate::runs_repo;

    use super::{get_run_artifact, list_run_artifacts_for_job, upsert_run_artifact_from_successful_run};

    #[tokio::test]
    async fn upsert_from_successful_run_records_snapshot_and_metrics() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let job = jobs_repo::create_job(
            &pool,
            "job",
            None,
            None,
            Some("UTC"),
            OverlapPolicy::Queue,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "root": "/" },
                "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .unwrap();

        let run = runs_repo::create_run(
            &pool,
            &job.id,
            runs_repo::RunStatus::Queued,
            1,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        runs_repo::set_run_target_snapshot(
            &pool,
            &run.id,
            serde_json::json!({
                "node_id": "hub",
                "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .unwrap();

        let progress = ProgressSnapshotV1 {
            v: 1,
            kind: ProgressKindV1::Backup,
            stage: "upload".to_string(),
            ts: 2,
            done: ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: 100,
            },
            total: Some(ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: 100,
            }),
            rate_bps: None,
            eta_seconds: None,
            detail: Some(serde_json::json!({
                "backup": {
                    "source_total": { "files": 10, "dirs": 2, "bytes": 123 },
                    "transfer_total_bytes": 100,
                    "transfer_done_bytes": 100
                }
            })),
        };
        runs_repo::set_run_progress(&pool, &run.id, Some(serde_json::to_value(progress).unwrap()))
            .await
            .unwrap();

        runs_repo::complete_run(
            &pool,
            &run.id,
            runs_repo::RunStatus::Success,
            Some(serde_json::json!({ "artifact_format": "archive_v1" })),
            None,
        )
        .await
        .unwrap();

        assert!(upsert_run_artifact_from_successful_run(&pool, &run.id)
            .await
            .unwrap());

        let got = get_run_artifact(&pool, &run.id).await.unwrap().unwrap();
        assert_eq!(got.job_id, job.id);
        assert_eq!(got.node_id, "hub");
        assert_eq!(got.target_type, "local_dir");
        assert_eq!(got.artifact_format, "archive_v1");
        assert_eq!(got.status, "present");
        assert_eq!(got.source_files, Some(10));
        assert_eq!(got.source_dirs, Some(2));
        assert_eq!(got.source_bytes, Some(123));
        assert_eq!(got.transfer_bytes, Some(100));

        let list = list_run_artifacts_for_job(&pool, &job.id, 0, 50, None)
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].run_id, run.id);
    }
}
