use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bastion_core::manifest::HashAlgorithm;
use bastion_core::progress::ProgressUnitsV1;
use bastion_storage::runs_repo;

use super::{entries_index, parts, unpack};

#[derive(Debug)]
pub(super) struct VerifyResult {
    pub(super) ok: bool,
    pub(super) files_total: u64,
    pub(super) files_ok: u64,
    pub(super) files_failed: u64,
    pub(super) sample_errors: Vec<String>,
}

pub(super) fn verify_restored(
    entries_path: &Path,
    restore_dir: &Path,
    expected_count: u64,
    on_progress: Option<&dyn Fn(ProgressUnitsV1)>,
) -> Result<VerifyResult, anyhow::Error> {
    const VERIFY_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

    let raw = std::fs::read(entries_path)?;
    let decoded = zstd::decode_all(std::io::Cursor::new(raw))?;
    let mut errors = Vec::<String>::new();
    let mut files_total = 0u64;
    let mut files_ok = 0u64;
    let mut files_failed = 0u64;
    let mut seen = 0u64;

    let mut progress_done = ProgressUnitsV1::default();
    let mut progress_last_emit = Instant::now();
    if let Some(cb) = on_progress {
        cb(progress_done);
    }

    for line in decoded.split(|b| *b == b'\n') {
        if line.is_empty() {
            continue;
        }
        seen += 1;
        let rec: entries_index::EntryRecord = serde_json::from_slice(line)?;
        if rec.kind != "file" {
            continue;
        }
        files_total += 1;
        progress_done.files = progress_done.files.saturating_add(1);
        progress_done.bytes = progress_done.bytes.saturating_add(rec.size);
        if let Some(cb) = on_progress
            && progress_last_emit.elapsed() >= VERIFY_PROGRESS_MIN_INTERVAL
        {
            progress_last_emit = Instant::now();
            cb(progress_done);
        }
        let rel = PathBuf::from(rec.path.replace('\\', "/"));
        let path = unpack::safe_join(restore_dir, &rel)
            .ok_or_else(|| anyhow::anyhow!("invalid restored path: {}", rec.path))?;
        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => {
                files_failed += 1;
                if errors.len() < 10 {
                    errors.push(format!("missing file: {}", rec.path));
                }
                continue;
            }
        };
        if meta.len() != rec.size {
            files_failed += 1;
            if errors.len() < 10 {
                errors.push(format!("size mismatch: {}", rec.path));
            }
            continue;
        }

        match (rec.hash_alg, rec.hash) {
            (Some(HashAlgorithm::Blake3), Some(expected_hash)) => {
                let computed = parts::hash_file_blake3(&path)?;
                if computed != expected_hash {
                    files_failed += 1;
                    if errors.len() < 10 {
                        errors.push(format!("hash mismatch: {}", rec.path));
                    }
                } else {
                    files_ok += 1;
                }
            }
            _ => {
                files_failed += 1;
                if errors.len() < 10 {
                    errors.push(format!("missing hash for: {}", rec.path));
                }
            }
        }
    }

    if let Some(cb) = on_progress {
        cb(progress_done);
    }

    if seen != expected_count && errors.len() < 10 {
        errors.push(format!(
            "entries_count mismatch: expected {}, got {}",
            expected_count, seen
        ));
    }

    Ok(VerifyResult {
        ok: files_failed == 0 && seen == expected_count,
        files_total,
        files_ok,
        files_failed,
        sample_errors: errors,
    })
}

pub(super) fn sqlite_paths_for_verify(run: &runs_repo::Run) -> Vec<String> {
    let Some(summary) = run.summary.as_ref() else {
        return Vec::new();
    };

    if let Some(name) = summary
        .get("sqlite")
        .and_then(|v| v.get("snapshot_name"))
        .and_then(|n| n.as_str())
    {
        return vec![name.to_string()];
    }

    if summary.get("vaultwarden").is_some() {
        return vec!["db.sqlite3".to_string()];
    }

    Vec::new()
}

#[derive(Debug)]
pub(super) struct SqliteVerifyResult {
    pub(super) ok: bool,
    pub(super) details: serde_json::Value,
}

pub(super) fn verify_sqlite_files(
    restore_dir: &Path,
    relative_paths: &[String],
) -> Result<SqliteVerifyResult, anyhow::Error> {
    if relative_paths.is_empty() {
        return Ok(SqliteVerifyResult {
            ok: true,
            details: serde_json::json!({ "skipped": true }),
        });
    }

    let mut results = Vec::<serde_json::Value>::new();
    let mut all_ok = true;
    for rel in relative_paths {
        let path = restore_dir.join(rel);
        match sqlite_integrity_check(&path) {
            Ok(check) => {
                all_ok &= check.ok;
                results.push(serde_json::json!({
                    "path": rel,
                    "ok": check.ok,
                    "truncated": check.truncated,
                    "lines": check.lines,
                }));
            }
            Err(error) => {
                all_ok = false;
                results.push(serde_json::json!({
                    "path": rel,
                    "ok": false,
                    "error": format!("{error:#}"),
                }));
            }
        }
    }

    Ok(SqliteVerifyResult {
        ok: all_ok,
        details: serde_json::json!({ "results": results }),
    })
}

#[derive(Debug)]
struct IntegrityCheck {
    ok: bool,
    lines: Vec<String>,
    truncated: bool,
}

fn sqlite_integrity_check(path: &Path) -> Result<IntegrityCheck, anyhow::Error> {
    use rusqlite::{Connection, OpenFlags};

    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let mut stmt = conn.prepare("PRAGMA integrity_check")?;
    let mut rows = stmt.query([])?;

    let mut lines = Vec::<String>::new();
    let mut truncated = false;
    while let Some(row) = rows.next()? {
        let line: String = row.get(0)?;
        lines.push(line);
        if lines.len() >= 64 {
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
    use tempfile::TempDir;

    use bastion_storage::runs_repo::{Run, RunStatus};

    use super::{sqlite_paths_for_verify, verify_restored, verify_sqlite_files};

    fn run_with_summary(summary: Option<serde_json::Value>) -> Run {
        Run {
            id: "run1".to_string(),
            job_id: "job1".to_string(),
            status: RunStatus::Success,
            started_at: 1,
            ended_at: Some(2),
            progress: None,
            summary,
            error: None,
        }
    }

    #[test]
    fn sqlite_paths_for_verify_returns_empty_when_summary_missing() {
        let run = run_with_summary(None);
        assert!(sqlite_paths_for_verify(&run).is_empty());
    }

    #[test]
    fn sqlite_paths_for_verify_extracts_sqlite_snapshot_name() {
        let run = run_with_summary(Some(serde_json::json!({
            "sqlite": { "snapshot_name": "my.db" }
        })));
        assert_eq!(sqlite_paths_for_verify(&run), vec!["my.db".to_string()]);
    }

    #[test]
    fn sqlite_paths_for_verify_uses_vaultwarden_default() {
        let run = run_with_summary(Some(serde_json::json!({
            "vaultwarden": { "files": 1 }
        })));
        assert_eq!(
            sqlite_paths_for_verify(&run),
            vec!["db.sqlite3".to_string()]
        );
    }

    #[test]
    fn sqlite_paths_for_verify_prefers_sqlite_over_vaultwarden() {
        let run = run_with_summary(Some(serde_json::json!({
            "sqlite": { "snapshot_name": "my.db" },
            "vaultwarden": { "files": 1 }
        })));
        assert_eq!(sqlite_paths_for_verify(&run), vec!["my.db".to_string()]);
    }

    #[test]
    fn verify_sqlite_files_skips_when_no_paths() {
        let tmp = TempDir::new().unwrap();
        let res = verify_sqlite_files(tmp.path(), &[]).unwrap();
        assert!(res.ok);
        assert_eq!(res.details, serde_json::json!({ "skipped": true }));
    }

    #[test]
    fn verify_sqlite_files_reports_missing_file() {
        let tmp = TempDir::new().unwrap();
        let restore_dir = tmp.path();
        let rels = vec!["missing.sqlite3".to_string()];
        let res = verify_sqlite_files(restore_dir, &rels).unwrap();
        assert!(!res.ok);
        assert_eq!(res.details["results"][0]["path"], "missing.sqlite3");
        assert_eq!(res.details["results"][0]["ok"], false);
        assert!(res.details["results"][0]["error"].is_string());
    }

    #[test]
    fn verify_sqlite_files_ok_for_valid_db() {
        let tmp = TempDir::new().unwrap();
        let restore_dir = tmp.path();
        let rel = "db.sqlite3".to_string();
        let db_path = restore_dir.join(&rel);

        {
            use rusqlite::Connection;
            let conn = Connection::open(&db_path).unwrap();
            conn.execute("CREATE TABLE t (id INTEGER PRIMARY KEY)", [])
                .unwrap();
        }

        let res = verify_sqlite_files(restore_dir, std::slice::from_ref(&rel)).unwrap();
        assert!(res.ok);
        assert_eq!(res.details["results"][0]["path"], rel);
        assert_eq!(res.details["results"][0]["ok"], true);
        assert_eq!(res.details["results"][0]["lines"][0], "ok");
        assert_eq!(res.details["results"][0]["truncated"], false);
    }

    #[test]
    fn verify_restored_ok_for_single_file() {
        let tmp = TempDir::new().unwrap();
        let restore_dir = tmp.path().join("restore");
        std::fs::create_dir_all(restore_dir.join("dir")).unwrap();
        let file_path = restore_dir.join("dir/file.txt");
        std::fs::write(&file_path, b"hello").unwrap();

        let hash = super::super::parts::hash_file_blake3(&file_path).unwrap();

        let entries_lines = format!(
            "{}\n{}\n",
            serde_json::json!({
                "path": "dir/file.txt",
                "kind": "file",
                "size": 5,
                "hash_alg": "blake3",
                "hash": hash,
            }),
            serde_json::json!({
                "path": "dir",
                "kind": "dir",
                "size": 0,
            }),
        );

        let entries_path = tmp.path().join("entries_index.jsonl.zst");
        let encoded = zstd::encode_all(entries_lines.as_bytes(), 0).unwrap();
        std::fs::write(&entries_path, encoded).unwrap();

        let res = verify_restored(&entries_path, &restore_dir, 2, None).unwrap();
        assert!(res.ok);
        assert_eq!(res.files_total, 1);
        assert_eq!(res.files_ok, 1);
        assert_eq!(res.files_failed, 0);
        assert!(res.sample_errors.is_empty());
    }

    #[test]
    fn verify_restored_reports_missing_file() {
        let tmp = TempDir::new().unwrap();
        let restore_dir = tmp.path().join("restore");
        std::fs::create_dir_all(&restore_dir).unwrap();

        let entries_lines = format!(
            "{}\n",
            serde_json::json!({
                "path": "missing.txt",
                "kind": "file",
                "size": 1,
                "hash_alg": "blake3",
                "hash": "deadbeef",
            })
        );

        let entries_path = tmp.path().join("entries_index.jsonl.zst");
        let encoded = zstd::encode_all(entries_lines.as_bytes(), 0).unwrap();
        std::fs::write(&entries_path, encoded).unwrap();

        let res = verify_restored(&entries_path, &restore_dir, 1, None).unwrap();
        assert!(!res.ok);
        assert_eq!(res.files_total, 1);
        assert_eq!(res.files_ok, 0);
        assert_eq!(res.files_failed, 1);
        assert!(res.sample_errors.iter().any(|e| e.contains("missing file")));
    }
}
