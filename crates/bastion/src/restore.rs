use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use bastion_core::manifest::{HashAlgorithm, ManifestV1};
use serde::Deserialize;
use sqlx::SqlitePool;
use url::Url;

use crate::job_spec;
use crate::operations_repo;
use crate::runs_repo;
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;
use crate::webdav::{WebdavClient, WebdavCredentials};

#[derive(Debug, Clone, Copy)]
pub enum ConflictPolicy {
    Overwrite,
    Skip,
    Fail,
}

impl ConflictPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::Skip => "skip",
            Self::Fail => "fail",
        }
    }
}

impl std::str::FromStr for ConflictPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "overwrite" => Ok(Self::Overwrite),
            "skip" => Ok(Self::Skip),
            "fail" => Ok(Self::Fail),
            _ => Err(anyhow::anyhow!("invalid conflict policy")),
        }
    }
}

#[derive(Debug, Deserialize)]
struct EntryRecord {
    path: String,
    kind: String,
    size: u64,
    hash_alg: Option<HashAlgorithm>,
    hash: Option<String>,
}

#[derive(Debug)]
enum TargetAccess {
    Webdav { client: WebdavClient, run_url: Url },
    LocalDir { run_dir: PathBuf },
}

#[derive(Debug, Clone)]
enum PayloadDecryption {
    None,
    AgeX25519 { identity: String },
}

async fn resolve_payload_decryption(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    manifest: &ManifestV1,
) -> Result<PayloadDecryption, anyhow::Error> {
    match manifest.pipeline.encryption.as_str() {
        "none" => Ok(PayloadDecryption::None),
        "age" => {
            let key_name = manifest
                .pipeline
                .encryption_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("missing manifest.pipeline.encryption_key"))?;

            let identity = crate::backup_encryption::get_age_identity(db, secrets, key_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing backup age identity: {}", key_name))?;
            Ok(PayloadDecryption::AgeX25519 { identity })
        }
        other => anyhow::bail!("unsupported manifest.pipeline.encryption: {}", other),
    }
}

pub async fn spawn_restore_operation(
    db: SqlitePool,
    secrets: std::sync::Arc<SecretsCrypto>,
    data_dir: PathBuf,
    op_id: String,
    run_id: String,
    destination_dir: PathBuf,
    conflict: ConflictPolicy,
) {
    tokio::spawn(async move {
        if let Err(error) = restore_operation(
            &db,
            &secrets,
            &data_dir,
            &op_id,
            &run_id,
            &destination_dir,
            conflict,
        )
        .await
        {
            let msg = format!("{error:#}");
            let _ = operations_repo::append_event(&db, &op_id, "error", "failed", &msg, None).await;
            let _ = operations_repo::complete_operation(
                &db,
                &op_id,
                operations_repo::OperationStatus::Failed,
                None,
                Some(&msg),
            )
            .await;
            let _ = tokio::fs::remove_dir_all(operation_dir(&data_dir, &op_id)).await;
        }
    });
}

pub async fn spawn_verify_operation(
    db: SqlitePool,
    secrets: std::sync::Arc<SecretsCrypto>,
    data_dir: PathBuf,
    op_id: String,
    run_id: String,
) {
    tokio::spawn(async move {
        if let Err(error) = verify_operation(&db, &secrets, &data_dir, &op_id, &run_id).await {
            let msg = format!("{error:#}");
            let _ = operations_repo::append_event(&db, &op_id, "error", "failed", &msg, None).await;
            let _ = operations_repo::complete_operation(
                &db,
                &op_id,
                operations_repo::OperationStatus::Failed,
                None,
                Some(&msg),
            )
            .await;
            let _ = tokio::fs::remove_dir_all(operation_dir(&data_dir, &op_id)).await;
        }
    });
}

async fn restore_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
    destination_dir: &Path,
    conflict: ConflictPolicy,
) -> Result<(), anyhow::Error> {
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;

    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = crate::jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    let access = open_target_access(db, secrets, &run.job_id, run_id, target_ref(&spec)).await?;
    ensure_complete(&access).await?;

    let op_dir = operation_dir(data_dir, op_id);
    tokio::fs::create_dir_all(op_dir.join("staging")).await?;

    let manifest = read_manifest(&access).await?;
    operations_repo::append_event(
        db,
        op_id,
        "info",
        "manifest",
        "manifest",
        Some(serde_json::json!({
            "artifacts": manifest.artifacts.len(),
            "entries_count": manifest.entry_index.count,
        })),
    )
    .await?;

    let decryption = resolve_payload_decryption(db, secrets, &manifest).await?;

    let staging_dir = op_dir.join("staging");
    let parts = fetch_parts(&access, &manifest, &staging_dir).await?;

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let dest = destination_dir.to_path_buf();
    let summary = tokio::task::spawn_blocking(move || {
        restore_from_parts(&parts, &dest, conflict, decryption)?;
        Ok::<_, anyhow::Error>(serde_json::json!({
            "destination_dir": dest.to_string_lossy().to_string(),
            "conflict_policy": conflict.as_str(),
        }))
    })
    .await??;

    operations_repo::append_event(db, op_id, "info", "complete", "complete", None).await?;
    operations_repo::complete_operation(
        db,
        op_id,
        operations_repo::OperationStatus::Success,
        Some(summary),
        None,
    )
    .await?;

    let _ = tokio::fs::remove_dir_all(&op_dir).await;

    Ok(())
}

async fn verify_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
) -> Result<(), anyhow::Error> {
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;

    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = crate::jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    let access = open_target_access(db, secrets, &run.job_id, run_id, target_ref(&spec)).await?;
    ensure_complete(&access).await?;

    let op_dir = operation_dir(data_dir, op_id);
    let staging_dir = op_dir.join("staging");
    tokio::fs::create_dir_all(&staging_dir).await?;

    let manifest = read_manifest(&access).await?;
    operations_repo::append_event(
        db,
        op_id,
        "info",
        "manifest",
        "manifest",
        Some(serde_json::json!({
            "artifacts": manifest.artifacts.len(),
            "entries_count": manifest.entry_index.count,
        })),
    )
    .await?;

    let decryption = resolve_payload_decryption(db, secrets, &manifest).await?;

    let entries_path = fetch_entries_index(&access, &staging_dir).await?;
    let parts = fetch_parts(&access, &manifest, &staging_dir).await?;

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let temp_restore_dir = op_dir.join("restore");
    tokio::fs::create_dir_all(&temp_restore_dir).await?;

    let record_count = manifest.entry_index.count;
    let sqlite_paths = sqlite_paths_for_verify(&run);

    let result = tokio::task::spawn_blocking(move || {
        restore_from_parts(
            &parts,
            &temp_restore_dir,
            ConflictPolicy::Overwrite,
            decryption,
        )?;
        let verify = verify_restored(&entries_path, &temp_restore_dir, record_count)?;

        let sqlite_results = verify_sqlite_files(&temp_restore_dir, &sqlite_paths)?;
        Ok::<_, anyhow::Error>((verify, sqlite_results))
    })
    .await??;

    let verify = result.0;
    let sqlite_results = result.1;

    operations_repo::append_event(
        db,
        op_id,
        if verify.ok && sqlite_results.ok {
            "info"
        } else {
            "error"
        },
        "verify",
        "verify",
        Some(serde_json::json!({
            "files_total": verify.files_total,
            "files_ok": verify.files_ok,
            "files_failed": verify.files_failed,
            "sample_errors": verify.sample_errors,
            "sqlite": sqlite_results.details,
        })),
    )
    .await?;

    let summary = serde_json::json!({
        "ok": verify.ok && sqlite_results.ok,
        "files_total": verify.files_total,
        "files_ok": verify.files_ok,
        "files_failed": verify.files_failed,
        "sqlite_ok": sqlite_results.ok,
        "sqlite": sqlite_results.details,
    });

    operations_repo::complete_operation(
        db,
        op_id,
        if verify.ok && sqlite_results.ok {
            operations_repo::OperationStatus::Success
        } else {
            operations_repo::OperationStatus::Failed
        },
        Some(summary),
        None,
    )
    .await?;

    let _ = tokio::fs::remove_dir_all(&op_dir).await;

    Ok(())
}

fn operation_dir(data_dir: &Path, op_id: &str) -> PathBuf {
    data_dir.join("operations").join(op_id)
}

fn target_ref(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

async fn open_target_access(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    job_id: &str,
    run_id: &str,
    target: &job_spec::TargetV1,
) -> Result<TargetAccess, anyhow::Error> {
    match target {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let cred_bytes = secrets_repo::get_secret(db, secrets, "webdav", secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;

            let mut base_url = Url::parse(base_url)?;
            if !base_url.path().ends_with('/') {
                base_url.set_path(&format!("{}/", base_url.path()));
            }
            let client = WebdavClient::new(base_url.clone(), credentials)?;

            let job_url = base_url.join(&format!("{job_id}/"))?;
            let run_url = job_url.join(&format!("{run_id}/"))?;
            Ok(TargetAccess::Webdav { client, run_url })
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let run_dir = PathBuf::from(base_dir.trim()).join(job_id).join(run_id);
            Ok(TargetAccess::LocalDir { run_dir })
        }
    }
}

async fn ensure_complete(access: &TargetAccess) -> Result<(), anyhow::Error> {
    match access {
        TargetAccess::Webdav { client, run_url } => {
            let url = run_url.join(crate::backup::COMPLETE_NAME)?;
            let exists = client.head_size(&url).await?.is_some();
            if !exists {
                anyhow::bail!("complete.json not found");
            }
        }
        TargetAccess::LocalDir { run_dir } => {
            let path = run_dir.join(crate::backup::COMPLETE_NAME);
            if !path.exists() {
                anyhow::bail!("complete.json not found");
            }
        }
    }
    Ok(())
}

async fn read_manifest(access: &TargetAccess) -> Result<ManifestV1, anyhow::Error> {
    let bytes = match access {
        TargetAccess::Webdav { client, run_url } => {
            let url = run_url.join(crate::backup::MANIFEST_NAME)?;
            client.get_bytes(&url).await?
        }
        TargetAccess::LocalDir { run_dir } => {
            tokio::fs::read(run_dir.join(crate::backup::MANIFEST_NAME)).await?
        }
    };

    Ok(serde_json::from_slice::<ManifestV1>(&bytes)?)
}

async fn fetch_entries_index(
    access: &TargetAccess,
    staging_dir: &Path,
) -> Result<PathBuf, anyhow::Error> {
    let dst = staging_dir.join(crate::backup::ENTRIES_INDEX_NAME);
    match access {
        TargetAccess::Webdav { client, run_url } => {
            let url = run_url.join(crate::backup::ENTRIES_INDEX_NAME)?;
            let expected = client.head_size(&url).await?;
            if let Some(size) = expected {
                if let Ok(meta) = tokio::fs::metadata(&dst).await {
                    if meta.len() == size {
                        return Ok(dst);
                    }
                }
            }
            client.get_to_file(&url, &dst, expected, 3).await?;
            Ok(dst)
        }
        TargetAccess::LocalDir { run_dir } => Ok(run_dir.join(crate::backup::ENTRIES_INDEX_NAME)),
    }
}

async fn fetch_parts(
    access: &TargetAccess,
    manifest: &ManifestV1,
    staging_dir: &Path,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut parts = Vec::with_capacity(manifest.artifacts.len());
    for part in &manifest.artifacts {
        match access {
            TargetAccess::Webdav { client, run_url } => {
                let dst = staging_dir.join(&part.name);
                if let Ok(meta) = tokio::fs::metadata(&dst).await {
                    if meta.len() == part.size {
                        parts.push(dst);
                        continue;
                    }
                }

                let url = run_url.join(&part.name)?;
                client.get_to_file(&url, &dst, Some(part.size), 3).await?;
                parts.push(dst);
            }
            TargetAccess::LocalDir { run_dir } => {
                parts.push(run_dir.join(&part.name));
            }
        }
    }

    // Verify part hashes (blocking).
    let expected = manifest
        .artifacts
        .iter()
        .map(|p| (p.size, p.hash_alg.clone(), p.hash.clone()))
        .collect::<Vec<_>>();
    let parts_clone = parts.clone();
    tokio::task::spawn_blocking(move || verify_parts(&parts_clone, &expected)).await??;

    Ok(parts)
}

fn verify_parts(
    parts: &[PathBuf],
    expected: &[(u64, HashAlgorithm, String)],
) -> Result<(), anyhow::Error> {
    for (idx, path) in parts.iter().enumerate() {
        let (size, alg, hash) = expected
            .get(idx)
            .ok_or_else(|| anyhow::anyhow!("missing expected part info"))?;
        let meta = std::fs::metadata(path)?;
        if meta.len() != *size {
            anyhow::bail!(
                "part size mismatch for {}: expected {}, got {}",
                path.display(),
                size,
                meta.len()
            );
        }
        match alg {
            HashAlgorithm::Blake3 => {
                let computed = hash_file_blake3(path)?;
                if &computed != hash {
                    anyhow::bail!(
                        "part hash mismatch for {}: expected {}, got {}",
                        path.display(),
                        hash,
                        computed
                    );
                }
            }
            other => anyhow::bail!("unsupported part hash algorithm: {other:?}"),
        }
    }
    Ok(())
}

fn restore_from_parts(
    part_paths: &[PathBuf],
    destination_dir: &Path,
    conflict: ConflictPolicy,
    decryption: PayloadDecryption,
) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(destination_dir)?;

    let files = part_paths
        .iter()
        .map(|p| File::open(p))
        .collect::<Result<Vec<_>, _>>()?;
    let reader = ConcatReader { files, index: 0 };
    let reader: Box<dyn Read> = match decryption {
        PayloadDecryption::None => Box::new(reader),
        PayloadDecryption::AgeX25519 { identity } => {
            use std::str::FromStr as _;

            let identity =
                age::x25519::Identity::from_str(identity.trim()).map_err(|e| anyhow::anyhow!(e))?;
            let decryptor = age::Decryptor::new(reader)?;
            let reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))?;
            Box::new(reader)
        }
    };
    let decoder = zstd::Decoder::new(reader)?;
    let mut archive = tar::Archive::new(decoder);
    archive.set_unpack_xattrs(false);
    archive.set_preserve_mtime(true);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let rel = entry.path()?.to_path_buf();
        let dest_path = safe_join(destination_dir, &rel)
            .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel.display()))?;

        match conflict {
            ConflictPolicy::Overwrite => {
                if dest_path.exists() {
                    remove_existing_path(&dest_path)?;
                }
            }
            ConflictPolicy::Skip => {
                if dest_path.exists() {
                    continue;
                }
            }
            ConflictPolicy::Fail => {
                if dest_path.exists() {
                    anyhow::bail!("restore conflict: {} exists", dest_path.display());
                }
            }
        }

        entry.unpack_in(destination_dir)?;
    }

    Ok(())
}

fn safe_join(base: &Path, rel: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::from(base);
    for c in rel.components() {
        match c {
            std::path::Component::Normal(p) => out.push(p),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn remove_existing_path(path: &Path) -> Result<(), anyhow::Error> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

struct ConcatReader {
    files: Vec<File>,
    index: usize,
}

impl Read for ConcatReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.index >= self.files.len() {
                return Ok(0);
            }
            let n = self.files[self.index].read(buf)?;
            if n == 0 {
                self.index += 1;
                continue;
            }
            return Ok(n);
        }
    }
}

fn hash_file_blake3(path: &Path) -> Result<String, anyhow::Error> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[derive(Debug)]
struct VerifyResult {
    ok: bool,
    files_total: u64,
    files_ok: u64,
    files_failed: u64,
    sample_errors: Vec<String>,
}

fn verify_restored(
    entries_path: &Path,
    restore_dir: &Path,
    expected_count: u64,
) -> Result<VerifyResult, anyhow::Error> {
    let raw = std::fs::read(entries_path)?;
    let decoded = zstd::decode_all(std::io::Cursor::new(raw))?;
    let mut errors = Vec::<String>::new();
    let mut files_total = 0u64;
    let mut files_ok = 0u64;
    let mut files_failed = 0u64;
    let mut seen = 0u64;

    for line in decoded.split(|b| *b == b'\n') {
        if line.is_empty() {
            continue;
        }
        seen += 1;
        let rec: EntryRecord = serde_json::from_slice(line)?;
        if rec.kind != "file" {
            continue;
        }
        files_total += 1;
        let rel = PathBuf::from(rec.path.replace('\\', "/"));
        let path = safe_join(restore_dir, &rel)
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
                let computed = hash_file_blake3(&path)?;
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

    if seen != expected_count {
        if errors.len() < 10 {
            errors.push(format!(
                "entries_count mismatch: expected {}, got {}",
                expected_count, seen
            ));
        }
    }

    Ok(VerifyResult {
        ok: files_failed == 0 && seen == expected_count,
        files_total,
        files_ok,
        files_failed,
        sample_errors: errors,
    })
}

fn sqlite_paths_for_verify(run: &runs_repo::Run) -> Vec<String> {
    let Some(summary) = run.summary.as_ref() else {
        return Vec::new();
    };

    if let Some(v) = summary.get("sqlite") {
        if let Some(name) = v.get("snapshot_name").and_then(|n| n.as_str()) {
            return vec![name.to_string()];
        }
    }

    if summary.get("vaultwarden").is_some() {
        return vec!["db.sqlite3".to_string()];
    }

    Vec::new()
}

#[derive(Debug)]
struct SqliteVerifyResult {
    ok: bool,
    details: serde_json::Value,
}

fn verify_sqlite_files(
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
    use std::fs::File;
    use std::path::Path;

    use tempfile::tempdir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::{ConflictPolicy, PayloadDecryption, restore_from_parts, safe_join};
    use crate::backup::PayloadEncryption;
    use crate::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};

    #[test]
    fn safe_join_rejects_parent() {
        let base = Path::new("/tmp");
        assert!(safe_join(base, Path::new("../etc")).is_none());
    }

    #[test]
    fn restore_from_parts_extracts_tar_zstd() {
        // Build a small tar.zst split into one part.
        let tmp = tempdir().unwrap();
        let part = tmp.path().join("payload.part000001");

        let file = File::create(&part).unwrap();
        let mut encoder = zstd::Encoder::new(file, 3).unwrap();
        {
            let mut tar = tar::Builder::new(&mut encoder);
            let src = tmp.path().join("hello.txt");
            std::fs::write(&src, b"hi").unwrap();
            tar.append_path_with_name(&src, Path::new("hello.txt"))
                .unwrap();
            tar.finish().unwrap();
        }
        encoder.finish().unwrap();

        let dest = tmp.path().join("out");
        restore_from_parts(
            &vec![part],
            &dest,
            ConflictPolicy::Overwrite,
            PayloadDecryption::None,
        )
        .unwrap();
        let out = std::fs::read(dest.join("hello.txt")).unwrap();
        assert_eq!(out, b"hi");
    }

    #[test]
    fn restore_from_parts_extracts_tar_zstd_age() {
        use age::secrecy::ExposeSecret as _;

        let tmp = tempdir().unwrap();
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let src_root = tmp.path().join("src");
        std::fs::create_dir_all(&src_root).unwrap();
        std::fs::write(src_root.join("hello.txt"), b"hi").unwrap();

        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        let identity_str = identity.to_string().expose_secret().to_string();

        let encryption = PayloadEncryption::AgeX25519 {
            recipient,
            key_name: "k".to_string(),
        };

        let job_id = Uuid::new_v4().to_string();
        let run_id = Uuid::new_v4().to_string();
        let source = FilesystemSource {
            root: src_root.to_string_lossy().to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = crate::backup::filesystem::build_filesystem_run(
            &data_dir,
            &job_id,
            &run_id,
            OffsetDateTime::now_utc(),
            &source,
            &encryption,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);

        let manifest_bytes = std::fs::read(&build.artifacts.manifest_path).unwrap();
        let manifest =
            serde_json::from_slice::<bastion_core::manifest::ManifestV1>(&manifest_bytes).unwrap();
        assert_eq!(manifest.pipeline.encryption, "age");
        assert_eq!(manifest.pipeline.encryption_key.as_deref(), Some("k"));

        let part_paths = build
            .artifacts
            .parts
            .iter()
            .map(|p| p.path.clone())
            .collect::<Vec<_>>();

        let dest = tmp.path().join("out_age");
        restore_from_parts(
            &part_paths,
            &dest,
            ConflictPolicy::Overwrite,
            PayloadDecryption::AgeX25519 {
                identity: identity_str,
            },
        )
        .unwrap();

        let out = std::fs::read(dest.join("hello.txt")).unwrap();
        assert_eq!(out, b"hi");
    }
}
