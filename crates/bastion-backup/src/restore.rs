use std::fs::File;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};

use bastion_core::HUB_NODE_ID;
use bastion_core::manifest::{HashAlgorithm, ManifestV1};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, info, warn};
use url::Url;

use bastion_core::job_spec;
use bastion_storage::operations_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::{WebdavClient, WebdavCredentials};

fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let _ = redacted.set_username("");
    let _ = redacted.set_password(None);
    redacted.set_query(None);
    redacted.set_fragment(None);
    redacted.to_string()
}

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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct RestoreSelection {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub dirs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EntryRecord {
    path: String,
    kind: String,
    size: u64,
    hash_alg: Option<HashAlgorithm>,
    hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunEntriesChild {
    pub path: String,
    pub kind: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct RunEntriesChildrenResponse {
    pub prefix: String,
    pub cursor: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<u64>,
    pub entries: Vec<RunEntriesChild>,
}

pub async fn list_run_entries_children(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    run_id: &str,
    prefix: Option<&str>,
    cursor: u64,
    limit: u64,
) -> Result<RunEntriesChildrenResponse, anyhow::Error> {
    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = bastion_storage::jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let node_id = job.agent_id.as_deref().unwrap_or(HUB_NODE_ID);
    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    let access =
        open_target_access(db, secrets, node_id, &run.job_id, run_id, target_ref(&spec)).await?;
    ensure_complete(&access).await?;

    let cache_dir = data_dir.join("cache").join("entries").join(run_id);
    tokio::fs::create_dir_all(&cache_dir).await?;
    let entries_path = fetch_entries_index(&access, &cache_dir).await?;

    let prefix = prefix.unwrap_or("").trim();
    let prefix = prefix
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();
    let limit = limit.clamp(1, 1000) as usize;
    let cursor = cursor as usize;

    let entries = tokio::task::spawn_blocking(move || {
        list_children_from_entries_index(&entries_path, &prefix, cursor, limit)
    })
    .await??;

    Ok(entries)
}

#[derive(Debug)]
enum TargetAccess {
    Webdav {
        client: Box<WebdavClient>,
        run_url: Url,
    },
    LocalDir {
        run_dir: PathBuf,
    },
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

#[allow(clippy::too_many_arguments)]
pub async fn spawn_restore_operation(
    db: SqlitePool,
    secrets: std::sync::Arc<SecretsCrypto>,
    data_dir: PathBuf,
    op_id: String,
    run_id: String,
    destination_dir: PathBuf,
    conflict: ConflictPolicy,
    selection: Option<RestoreSelection>,
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
            selection,
        )
        .await
        {
            warn!(
                op_id = %op_id,
                run_id = %run_id,
                destination_dir = %destination_dir.display(),
                error = %error,
                "restore operation failed"
            );
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
            warn!(op_id = %op_id, run_id = %run_id, error = %error, "verify operation failed");
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

#[allow(clippy::too_many_arguments)]
async fn restore_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
    destination_dir: &Path,
    conflict: ConflictPolicy,
    selection: Option<RestoreSelection>,
) -> Result<(), anyhow::Error> {
    info!(
        op_id = %op_id,
        run_id = %run_id,
        destination_dir = %destination_dir.display(),
        conflict = %conflict.as_str(),
        selection_files = selection.as_ref().map(|s| s.files.len()).unwrap_or(0),
        selection_dirs = selection.as_ref().map(|s| s.dirs.len()).unwrap_or(0),
        "restore operation started"
    );
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;

    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = bastion_storage::jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let node_id = job.agent_id.as_deref().unwrap_or(HUB_NODE_ID);
    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    let access =
        open_target_access(db, secrets, node_id, &run.job_id, run_id, target_ref(&spec)).await?;
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
    info!(
        op_id = %op_id,
        run_id = %run_id,
        parts_count = parts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for restore"
    );

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let dest = destination_dir.to_path_buf();
    let selection = selection.clone();
    let summary = tokio::task::spawn_blocking(move || {
        restore_from_parts(&parts, &dest, conflict, decryption, selection.as_ref())?;
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

    info!(op_id = %op_id, run_id = %run_id, "restore operation completed");
    Ok(())
}

async fn verify_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
) -> Result<(), anyhow::Error> {
    info!(op_id = %op_id, run_id = %run_id, "verify operation started");
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;

    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = bastion_storage::jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let node_id = job.agent_id.as_deref().unwrap_or(HUB_NODE_ID);
    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    let access =
        open_target_access(db, secrets, node_id, &run.job_id, run_id, target_ref(&spec)).await?;
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
    info!(
        op_id = %op_id,
        run_id = %run_id,
        parts_count = parts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for verify"
    );

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
            None,
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

    info!(
        op_id = %op_id,
        run_id = %run_id,
        ok = verify.ok && sqlite_results.ok,
        "verify operation completed"
    );
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
    node_id: &str,
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
            let cred_bytes = secrets_repo::get_secret(db, secrets, node_id, "webdav", secret_name)
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
            debug!(
                job_id = %job_id,
                run_id = %run_id,
                target = "webdav",
                base_url = %redact_url(&base_url),
                run_url = %redact_url(&run_url),
                "resolved restore target access"
            );
            Ok(TargetAccess::Webdav {
                client: Box::new(client),
                run_url,
            })
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let run_dir = PathBuf::from(base_dir.trim()).join(job_id).join(run_id);
            debug!(
                job_id = %job_id,
                run_id = %run_id,
                target = "local_dir",
                run_dir = %run_dir.display(),
                "resolved restore target access"
            );
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
            if let Some(size) = expected
                && let Ok(meta) = tokio::fs::metadata(&dst).await
                && meta.len() == size
            {
                return Ok(dst);
            }
            client.get_to_file(&url, &dst, expected, 3).await?;
            Ok(dst)
        }
        TargetAccess::LocalDir { run_dir } => Ok(run_dir.join(crate::backup::ENTRIES_INDEX_NAME)),
    }
}

fn list_children_from_entries_index(
    entries_path: &Path,
    prefix: &str,
    cursor: usize,
    limit: usize,
) -> Result<RunEntriesChildrenResponse, anyhow::Error> {
    use std::collections::HashMap;

    #[derive(Debug)]
    struct ChildAgg {
        kind: String,
        size: u64,
    }

    let file = File::open(entries_path)?;
    let decoder = zstd::Decoder::new(file)?;
    let reader = std::io::BufReader::new(decoder);

    let prefix = prefix.trim().trim_start_matches('/').trim_end_matches('/');
    let prefix_slash = if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix}/")
    };

    let mut children = HashMap::<String, ChildAgg>::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let rec: EntryRecord = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let path = rec.path;
        let remainder: &str = if prefix.is_empty() {
            path.as_str()
        } else if path == prefix {
            continue;
        } else if let Some(rest) = path.strip_prefix(&prefix_slash) {
            rest
        } else {
            continue;
        };

        if remainder.is_empty() {
            continue;
        }

        let (child_name, has_more) = match remainder.split_once('/') {
            Some((first, _rest)) => (first, true),
            None => (remainder, false),
        };
        if child_name.is_empty() {
            continue;
        }

        let child_path = if prefix.is_empty() {
            child_name.to_string()
        } else {
            format!("{prefix}/{child_name}")
        };

        let inferred_dir = has_more;
        let kind = if inferred_dir {
            "dir".to_string()
        } else {
            rec.kind
        };
        let kind = if matches!(kind.as_str(), "file" | "dir" | "symlink") {
            kind
        } else if inferred_dir {
            "dir".to_string()
        } else {
            "file".to_string()
        };
        let size = if kind == "file" { rec.size } else { 0 };

        children
            .entry(child_path)
            .and_modify(|existing| {
                if existing.kind != "dir" && kind == "dir" {
                    existing.kind = "dir".to_string();
                    existing.size = 0;
                    return;
                }
                if existing.kind == kind && kind == "file" {
                    existing.size = existing.size.max(size);
                }
            })
            .or_insert(ChildAgg { kind, size });
    }

    let mut entries = children
        .into_iter()
        .map(|(path, agg)| RunEntriesChild {
            path,
            kind: agg.kind,
            size: agg.size,
        })
        .collect::<Vec<_>>();

    fn kind_rank(kind: &str) -> u8 {
        match kind {
            "dir" => 0,
            "file" => 1,
            "symlink" => 2,
            _ => 3,
        }
    }

    entries.sort_by(|a, b| {
        kind_rank(&a.kind)
            .cmp(&kind_rank(&b.kind))
            .then_with(|| a.path.cmp(&b.path))
    });

    let total = entries.len();
    let start = cursor.min(total);
    let end = start.saturating_add(limit).min(total);
    let next_cursor = if end < total { Some(end as u64) } else { None };
    let page = entries
        .into_iter()
        .skip(start)
        .take(limit)
        .collect::<Vec<_>>();

    Ok(RunEntriesChildrenResponse {
        prefix: prefix.to_string(),
        cursor: start as u64,
        next_cursor,
        entries: page,
    })
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
                if let Ok(meta) = tokio::fs::metadata(&dst).await
                    && meta.len() == part.size
                {
                    parts.push(dst);
                    continue;
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
    selection: Option<&RestoreSelection>,
) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(destination_dir)?;

    let selection = selection.map(normalize_restore_selection).transpose()?;

    let files = part_paths
        .iter()
        .map(File::open)
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

        let rel_match = archive_path_for_match(&rel)
            .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", rel.display()))?;
        if let Some(selection) = selection.as_ref()
            && !selection.matches(&rel_match)
        {
            continue;
        }

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

#[derive(Debug)]
struct NormalizedRestoreSelection {
    files: std::collections::HashSet<String>,
    dirs: Vec<String>,
}

impl NormalizedRestoreSelection {
    fn matches(&self, archive_path: &str) -> bool {
        if self.files.contains(archive_path) {
            return true;
        }
        for dir in &self.dirs {
            if archive_path == dir {
                return true;
            }
            if archive_path.starts_with(dir)
                && archive_path.as_bytes().get(dir.len()) == Some(&b'/')
            {
                return true;
            }
        }
        false
    }
}

fn normalize_restore_path(path: &str, allow_trailing_slash: bool) -> Option<String> {
    let mut s = path.trim().replace('\\', "/");
    if s.is_empty() {
        return None;
    }
    while s.starts_with("./") {
        s = s.trim_start_matches("./").to_string();
    }
    while s.starts_with('/') {
        s = s.trim_start_matches('/').to_string();
    }
    if !allow_trailing_slash {
        while s.ends_with('/') {
            s = s.trim_end_matches('/').to_string();
        }
    }
    let s = s.trim_matches('/').to_string();
    if s.is_empty() {
        return None;
    }
    if s.split('/').any(|seg| seg == "..") {
        return None;
    }
    Some(s)
}

fn normalize_restore_selection(
    selection: &RestoreSelection,
) -> Result<NormalizedRestoreSelection, anyhow::Error> {
    let mut files = std::collections::HashSet::<String>::new();
    let mut dirs = std::collections::HashSet::<String>::new();

    for f in &selection.files {
        if let Some(v) = normalize_restore_path(f, false) {
            files.insert(v);
        }
    }
    for d in &selection.dirs {
        if let Some(v) = normalize_restore_path(d, true) {
            dirs.insert(v.trim_end_matches('/').to_string());
        }
    }

    if files.is_empty() && dirs.is_empty() {
        anyhow::bail!("restore selection is empty");
    }

    let mut dirs = dirs.into_iter().collect::<Vec<_>>();
    dirs.sort_by_key(|v| std::cmp::Reverse(v.len())); // longest first for prefix checks
    Ok(NormalizedRestoreSelection { files, dirs })
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

fn archive_path_for_match(rel: &Path) -> Option<String> {
    let mut parts = Vec::<String>::new();
    for c in rel.components() {
        match c {
            std::path::Component::Normal(p) => parts.push(p.to_string_lossy().to_string()),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
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

fn sqlite_paths_for_verify(run: &runs_repo::Run) -> Vec<String> {
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
    use std::io::Write;
    use std::path::Path;

    use tempfile::tempdir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::{
        ConflictPolicy, PayloadDecryption, RestoreSelection, list_children_from_entries_index,
        restore_from_parts, safe_join,
    };
    use crate::backup::PayloadEncryption;
    use bastion_core::job_spec::{
        FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy,
    };

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
            &[part],
            &dest,
            ConflictPolicy::Overwrite,
            PayloadDecryption::None,
            None,
        )
        .unwrap();
        let out = std::fs::read(dest.join("hello.txt")).unwrap();
        assert_eq!(out, b"hi");
    }

    #[test]
    fn restore_from_parts_respects_selected_files() {
        let tmp = tempdir().unwrap();
        let part = tmp.path().join("payload.part000001");

        let file = File::create(&part).unwrap();
        let mut encoder = zstd::Encoder::new(file, 3).unwrap();
        {
            let mut tar = tar::Builder::new(&mut encoder);
            let a = tmp.path().join("a.txt");
            let b = tmp.path().join("b.txt");
            std::fs::write(&a, b"a").unwrap();
            std::fs::write(&b, b"b").unwrap();
            tar.append_path_with_name(&a, Path::new("a.txt")).unwrap();
            tar.append_path_with_name(&b, Path::new("b.txt")).unwrap();
            tar.finish().unwrap();
        }
        encoder.finish().unwrap();

        let dest = tmp.path().join("out_partial_file");
        let sel = RestoreSelection {
            files: vec!["a.txt".to_string()],
            dirs: vec![],
        };
        restore_from_parts(
            &[part],
            &dest,
            ConflictPolicy::Overwrite,
            PayloadDecryption::None,
            Some(&sel),
        )
        .unwrap();

        assert!(dest.join("a.txt").exists());
        assert!(!dest.join("b.txt").exists());
    }

    #[test]
    fn restore_from_parts_respects_selected_dirs() {
        let tmp = tempdir().unwrap();
        let part = tmp.path().join("payload.part000001");

        let file = File::create(&part).unwrap();
        let mut encoder = zstd::Encoder::new(file, 3).unwrap();
        {
            let mut tar = tar::Builder::new(&mut encoder);
            let a = tmp.path().join("a.txt");
            let b = tmp.path().join("b.txt");
            let c = tmp.path().join("c.txt");
            std::fs::write(&a, b"a").unwrap();
            std::fs::write(&b, b"b").unwrap();
            std::fs::write(&c, b"c").unwrap();
            tar.append_path_with_name(&a, Path::new("dir/a.txt"))
                .unwrap();
            tar.append_path_with_name(&b, Path::new("dir/b.txt"))
                .unwrap();
            tar.append_path_with_name(&c, Path::new("c.txt")).unwrap();
            tar.finish().unwrap();
        }
        encoder.finish().unwrap();

        let dest = tmp.path().join("out_partial_dir");
        let sel = RestoreSelection {
            files: vec![],
            dirs: vec!["dir".to_string()],
        };
        restore_from_parts(
            &[part],
            &dest,
            ConflictPolicy::Overwrite,
            PayloadDecryption::None,
            Some(&sel),
        )
        .unwrap();

        assert!(dest.join("dir").join("a.txt").exists());
        assert!(dest.join("dir").join("b.txt").exists());
        assert!(!dest.join("c.txt").exists());
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
            paths: Vec::new(),
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
            None,
        )
        .unwrap();

        let out = std::fs::read(dest.join("hello.txt")).unwrap();
        assert_eq!(out, b"hi");
    }

    #[test]
    fn entries_children_lists_unique_children() {
        #[derive(serde::Serialize)]
        struct Rec<'a> {
            path: &'a str,
            kind: &'a str,
            size: u64,
            hash_alg: Option<&'a str>,
            hash: Option<&'a str>,
        }

        let tmp = tempdir().unwrap();
        let entries_path = tmp.path().join("entries.jsonl.zst");

        let file = File::create(&entries_path).unwrap();
        let mut enc = zstd::Encoder::new(file, 3).unwrap();
        for rec in [
            Rec {
                path: "etc",
                kind: "dir",
                size: 0,
                hash_alg: None,
                hash: None,
            },
            Rec {
                path: "etc/hosts",
                kind: "file",
                size: 2,
                hash_alg: Some("blake3"),
                hash: Some("x"),
            },
            Rec {
                path: "etc/ssh/sshd_config",
                kind: "file",
                size: 3,
                hash_alg: Some("blake3"),
                hash: Some("y"),
            },
            Rec {
                path: "var/log/syslog",
                kind: "file",
                size: 4,
                hash_alg: Some("blake3"),
                hash: Some("z"),
            },
        ] {
            let line = serde_json::to_vec(&rec).unwrap();
            enc.write_all(&line).unwrap();
            enc.write_all(b"\n").unwrap();
        }
        enc.finish().unwrap();

        let root = list_children_from_entries_index(&entries_path, "", 0, 100).unwrap();
        assert_eq!(root.prefix, "");
        assert!(
            root.entries
                .iter()
                .any(|e| e.path == "etc" && e.kind == "dir")
        );
        assert!(
            root.entries
                .iter()
                .any(|e| e.path == "var" && e.kind == "dir")
        );

        let etc = list_children_from_entries_index(&entries_path, "etc", 0, 100).unwrap();
        assert!(
            etc.entries
                .iter()
                .any(|e| e.path == "etc/hosts" && e.kind == "file")
        );
        assert!(
            etc.entries
                .iter()
                .any(|e| e.path == "etc/ssh" && e.kind == "dir")
        );

        let ssh = list_children_from_entries_index(&entries_path, "etc/ssh", 0, 100).unwrap();
        assert!(
            ssh.entries
                .iter()
                .any(|e| e.path == "etc/ssh/sshd_config" && e.kind == "file")
        );
    }
}
