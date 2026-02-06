use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};
use time::OffsetDateTime;
use tracing::{info, warn};

use crate::backup::{BuildPipelineOptions, LocalArtifact, LocalRunArtifacts};
use bastion_core::job_spec::{
    FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy, SqliteSource,
};
const SQLITE_BACKUP_PAGES_PER_STEP: i32 = 100;
const SQLITE_BACKUP_PAUSE: Duration = Duration::from_millis(10);
const SQLITE_INTEGRITY_MAX_LINES: usize = 64;

#[derive(Debug, Clone)]
pub struct IntegrityCheck {
    pub ok: bool,
    pub lines: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug)]
pub struct SqliteRunArtifacts {
    pub artifacts: LocalRunArtifacts,
    pub snapshot_name: String,
    #[allow(dead_code)]
    pub snapshot_path: PathBuf,
    pub snapshot_size: u64,
    pub integrity_check: Option<IntegrityCheck>,
}

pub fn build_sqlite_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &SqliteSource,
    pipeline: BuildPipelineOptions<'_>,
    on_part_finished: Option<Box<dyn Fn(LocalArtifact) -> std::io::Result<()> + Send>>,
) -> Result<SqliteRunArtifacts, anyhow::Error> {
    let BuildPipelineOptions {
        artifact_format,
        encryption,
        part_size_bytes,
    } = pipeline;
    info!(
        job_id = %job_id,
        run_id = %run_id,
        source_path = %source.path,
        integrity_check = source.integrity_check,
        artifact_format = ?artifact_format,
        encryption = ?encryption,
        part_size_bytes,
        "building sqlite backup artifacts"
    );

    let run_dir = crate::backup::run_dir(data_dir, run_id);
    let source_dir = run_dir.join("source");
    std::fs::create_dir_all(&source_dir)?;

    let snapshot_name = snapshot_name(&source.path);
    let snapshot_path = source_dir.join(&snapshot_name);

    create_snapshot(&source.path, &snapshot_path)?;

    let snapshot_size = std::fs::metadata(&snapshot_path)?.len();

    let integrity_check = if source.integrity_check {
        let check = integrity_check(&snapshot_path)?;
        if !check.ok {
            warn!(
                job_id = %job_id,
                run_id = %run_id,
                snapshot_path = %snapshot_path.display(),
                truncated = check.truncated,
                "sqlite integrity_check reported problems"
            );
        }
        Some(check)
    } else {
        None
    };

    let fs_source = FilesystemSource {
        pre_scan: true,
        paths: Vec::new(),
        root: source_dir.to_string_lossy().to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
        snapshot_mode: Default::default(),
        snapshot_provider: None,
        consistency_policy: Default::default(),
        consistency_fail_threshold: None,
        upload_on_consistency_failure: None,
    };
    let build = crate::backup::filesystem::build_filesystem_run(
        data_dir,
        job_id,
        run_id,
        started_at,
        &fs_source,
        BuildPipelineOptions {
            artifact_format,
            encryption,
            part_size_bytes,
        },
        None,
        None,
        on_part_finished,
        None,
    )?;
    if build.issues.errors_total > 0 {
        anyhow::bail!(
            "unexpected filesystem issues while packaging sqlite snapshot: {}",
            build.issues.errors_total
        );
    }
    let artifacts = build.artifacts;

    info!(
        job_id = %job_id,
        run_id = %run_id,
        snapshot_name = %snapshot_name,
        snapshot_size,
        entries_count = artifacts.entries_count,
        parts_count = artifacts.parts.len(),
        parts_bytes = artifacts.parts.iter().map(|p| p.size).sum::<u64>(),
        "built sqlite backup artifacts"
    );

    Ok(SqliteRunArtifacts {
        artifacts,
        snapshot_name,
        snapshot_path,
        snapshot_size,
        integrity_check,
    })
}

fn snapshot_name(source_path: &str) -> String {
    Path::new(source_path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("database.sqlite3")
        .to_string()
}

pub(crate) fn create_snapshot(
    source_path: &str,
    snapshot_path: &Path,
) -> Result<(), anyhow::Error> {
    let src = Connection::open_with_flags(
        source_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let mut dst = Connection::open(snapshot_path)?;
    let backup = rusqlite::backup::Backup::new(&src, &mut dst)?;
    backup.run_to_completion(SQLITE_BACKUP_PAGES_PER_STEP, SQLITE_BACKUP_PAUSE, None)?;
    Ok(())
}

fn integrity_check(snapshot_path: &Path) -> Result<IntegrityCheck, anyhow::Error> {
    let conn = Connection::open_with_flags(
        snapshot_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let mut stmt = conn.prepare("PRAGMA integrity_check")?;
    let mut rows = stmt.query([])?;

    let mut lines = Vec::<String>::new();
    let mut truncated = false;
    while let Some(row) = rows.next()? {
        let line: String = row.get(0)?;
        lines.push(line);
        if lines.len() >= SQLITE_INTEGRITY_MAX_LINES {
            truncated = rows.next()?.is_some();
            break;
        }
    }

    let ok = lines.len() == 1 && lines[0] == "ok";
    Ok(IntegrityCheck {
        ok,
        lines,
        truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_sqlite_run, integrity_check};
    use crate::backup::{BuildPipelineOptions, PayloadEncryption};
    use bastion_core::job_spec::SqliteSource;
    use bastion_core::manifest::ArtifactFormatV1;
    use rusqlite::Connection;
    use tempfile::tempdir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[test]
    fn integrity_check_ok_for_valid_db() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db3");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE t(x INTEGER); INSERT INTO t VALUES(1);")
            .unwrap();
        drop(conn);

        let result = integrity_check(&path).unwrap();
        assert!(result.ok);
    }

    #[test]
    fn build_sqlite_run_creates_snapshot_and_artifacts() {
        let tmp = tempdir().unwrap();
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let source_db = tmp.path().join("source.db3");
        let conn = Connection::open(&source_db).unwrap();
        conn.execute_batch("CREATE TABLE foo(x INTEGER); INSERT INTO foo VALUES(42);")
            .unwrap();
        drop(conn);

        let job_id = Uuid::new_v4().to_string();
        let run_id = Uuid::new_v4().to_string();

        let source = SqliteSource {
            path: source_db.to_string_lossy().to_string(),
            integrity_check: true,
        };

        let encryption = PayloadEncryption::None;
        let result = build_sqlite_run(
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

        let snapshot_path = crate::backup::run_dir(&data_dir, &run_id)
            .join("source")
            .join(&result.snapshot_name);
        assert_eq!(result.snapshot_path, snapshot_path);
        assert!(snapshot_path.exists());
        assert!(result.snapshot_size > 0);
        assert_eq!(result.artifacts.entries_count, 1);
        assert!(result.integrity_check.as_ref().is_some_and(|r| r.ok));
        assert!(!result.artifacts.parts.is_empty());
        assert!(result.artifacts.manifest_path.exists());
        assert!(result.artifacts.complete_path.exists());
    }
}
