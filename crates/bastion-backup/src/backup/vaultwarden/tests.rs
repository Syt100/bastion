use super::build_vaultwarden_run;
use crate::backup::{BuildPipelineOptions, PayloadEncryption};
use bastion_core::job_spec::VaultwardenSource;
use bastion_core::manifest::ArtifactFormatV1;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;
use time::OffsetDateTime;
use uuid::Uuid;

fn read_entries(paths: &std::path::Path) -> Vec<serde_json::Value> {
    let raw = fs::read(paths).unwrap();
    let decoded = zstd::decode_all(std::io::Cursor::new(raw)).unwrap();
    decoded
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice::<serde_json::Value>(line).unwrap())
        .collect()
}

#[test]
fn vaultwarden_run_includes_snapshot_and_files() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let vw_dir = tmp.path().join("vw");
    fs::create_dir_all(vw_dir.join("attachments")).unwrap();
    fs::write(vw_dir.join("attachments").join("hello.txt"), b"hi").unwrap();

    let db_path = vw_dir.join("db.sqlite3");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("CREATE TABLE foo(x INTEGER); INSERT INTO foo VALUES(42);")
        .unwrap();
    drop(conn);

    fs::write(vw_dir.join("db.sqlite3-wal"), b"ignored").unwrap();

    let job_id = Uuid::new_v4().to_string();
    let run_id = Uuid::new_v4().to_string();

    let source = VaultwardenSource {
        data_dir: vw_dir.to_string_lossy().to_string(),
    };
    let encryption = PayloadEncryption::None;
    let build = build_vaultwarden_run(
        &data_dir,
        &job_id,
        &run_id,
        OffsetDateTime::now_utc(),
        &source,
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &encryption,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
    )
    .unwrap();
    assert!(build.consistency.is_empty());
    let artifacts = build.artifacts;

    let entries = read_entries(&artifacts.entries_index_path);
    let paths: Vec<String> = entries
        .iter()
        .filter_map(|v| {
            v.get("path")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    assert!(paths.contains(&"attachments/hello.txt".to_string()));
    assert!(paths.contains(&"db.sqlite3".to_string()));
    assert!(!paths.contains(&"db.sqlite3-wal".to_string()));

    let snapshot_path = artifacts.run_dir.join("source").join("db.sqlite3");
    let snapshot_conn = Connection::open(snapshot_path).unwrap();
    let n: i64 = snapshot_conn
        .query_row("SELECT x FROM foo", [], |row| row.get(0))
        .unwrap();
    assert_eq!(n, 42);
}

#[test]
fn vaultwarden_run_rejects_non_archive_v1_format() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let job_id = Uuid::new_v4().to_string();
    let run_id = Uuid::new_v4().to_string();
    let source = VaultwardenSource {
        data_dir: "/".to_string(),
    };
    let encryption = PayloadEncryption::None;

    let err = build_vaultwarden_run(
        &data_dir,
        &job_id,
        &run_id,
        OffsetDateTime::now_utc(),
        &source,
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::RawTreeV1,
            encryption: &encryption,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
    )
    .expect_err("expected format validation error");
    assert!(err.to_string().contains("support only archive_v1"));
}

#[test]
fn vaultwarden_run_requires_source_data_dir() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let job_id = Uuid::new_v4().to_string();
    let run_id = Uuid::new_v4().to_string();
    let source = VaultwardenSource {
        data_dir: "   ".to_string(),
    };
    let encryption = PayloadEncryption::None;

    let err = build_vaultwarden_run(
        &data_dir,
        &job_id,
        &run_id,
        OffsetDateTime::now_utc(),
        &source,
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &encryption,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
    )
    .expect_err("expected missing data_dir error");
    assert!(
        err.to_string()
            .contains("vaultwarden.source.data_dir is required")
    );
}
