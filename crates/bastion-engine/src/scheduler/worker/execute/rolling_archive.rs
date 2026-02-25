use std::path::Path;
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_core::manifest::ArtifactFormatV1;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavCredentials;

use bastion_backup as backup;
use bastion_targets as targets;

#[derive(Debug, Clone)]
struct RollingUploaderFailureSnapshot {
    source: Arc<anyhow::Error>,
    message: String,
    error_kind: Option<String>,
    error_code: Option<String>,
    hint: Option<String>,
    http_status: Option<u16>,
    attempt: Option<u32>,
    max_attempts: Option<u32>,
}

impl RollingUploaderFailureSnapshot {
    fn from_source(source: Arc<anyhow::Error>) -> Self {
        let mut out = Self {
            message: source.to_string(),
            source,
            error_kind: None,
            error_code: None,
            hint: None,
            http_status: None,
            attempt: None,
            max_attempts: None,
        };

        for cause in out.source.chain() {
            if let Some(put) = cause.downcast_ref::<targets::WebdavPutError>() {
                out.error_kind = Some(put.diagnostic.kind.as_str().to_string());
                out.error_code = Some(put.diagnostic.code.to_string());
                out.hint = Some(put.diagnostic.hint.to_string());
                out.http_status = put.diagnostic.http_status;
                out.attempt = Some(put.attempt);
                out.max_attempts = Some(put.max_attempts);
                return out;
            }

            if let Some(http) = cause.downcast_ref::<targets::WebdavHttpError>() {
                out.http_status = Some(http.status.as_u16());
            }
        }

        out
    }
}

#[derive(Debug)]
struct RollingUploaderTaskError {
    source: Arc<anyhow::Error>,
}

impl std::fmt::Display for RollingUploaderTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl std::error::Error for RollingUploaderTaskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref().as_ref())
    }
}

#[derive(Debug)]
pub(crate) struct RollingUploaderDroppedError {
    pub(crate) part_name: String,
    pub(crate) part_size_bytes: u64,
    pub(crate) uploader_message: Option<String>,
    pub(crate) uploader_error_kind: Option<String>,
    pub(crate) uploader_error_code: Option<String>,
    pub(crate) hint: Option<String>,
    pub(crate) http_status: Option<u16>,
    pub(crate) attempt: Option<u32>,
    pub(crate) max_attempts: Option<u32>,
    source: Option<Arc<anyhow::Error>>,
}

impl std::fmt::Display for RollingUploaderDroppedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rolling uploader dropped while finalizing {} (size={} bytes)",
            self.part_name, self.part_size_bytes
        )?;

        if let Some(kind) = self.uploader_error_kind.as_ref() {
            write!(f, ", kind={kind}")?;
        }
        if let Some(code) = self.uploader_error_code.as_ref() {
            write!(f, ", code={code}")?;
        }
        if let Some(status) = self.http_status {
            write!(f, ", http_status={status}")?;
        }
        if let (Some(attempt), Some(max_attempts)) = (self.attempt, self.max_attempts) {
            write!(f, ", attempt={attempt}/{max_attempts}")?;
        }

        if let Some(message) = self.uploader_message.as_ref() {
            write!(f, ": {message}")?;
        }
        if let Some(hint) = self.hint.as_ref() {
            write!(f, "; hint: {hint}")?;
        }

        Ok(())
    }
}

impl std::error::Error for RollingUploaderDroppedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|source| source.as_ref().as_ref())
    }
}

fn is_rolling_uploader_drop(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<RollingUploaderDroppedError>()
            .is_some()
    })
}

pub(super) fn merge_packaging_and_uploader_errors(
    packaging_error: anyhow::Error,
    uploader_error: anyhow::Error,
) -> anyhow::Error {
    if is_rolling_uploader_drop(&packaging_error) {
        return uploader_error.context(format!(
            "packaging aborted after rolling uploader failure: {packaging_error}"
        ));
    }

    packaging_error.context(format!("rolling uploader also failed: {uploader_error}"))
}

pub(super) async fn join_parts_uploader(
    handle: Option<tokio::task::JoinHandle<Result<(), anyhow::Error>>>,
) -> Result<(), anyhow::Error> {
    let Some(handle) = handle else {
        return Ok(());
    };

    match handle.await {
        Ok(result) => result,
        Err(error) => Err(anyhow::anyhow!(
            "rolling uploader task join failed: {error}"
        )),
    }
}

/// Prepare a rolling uploader for archive_v1 part files.
///
/// Returns:
/// - on_part_finished hook to pass into the archive builder
/// - a join handle that completes once all received parts have been stored (and local files deleted)
pub(super) async fn prepare_archive_part_uploader(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    target: &job_spec::TargetV1,
    job_id: &str,
    run_id: &str,
    artifact_format: ArtifactFormatV1,
) -> Result<
    (
        Option<Box<dyn Fn(backup::LocalArtifact) -> std::io::Result<()> + Send>>,
        Option<tokio::task::JoinHandle<Result<(), anyhow::Error>>>,
    ),
    anyhow::Error,
> {
    if artifact_format != ArtifactFormatV1::ArchiveV1 {
        return Ok((None, None));
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<backup::LocalArtifact>(1);
    let uploader_failure: Arc<Mutex<Option<RollingUploaderFailureSnapshot>>> =
        Arc::new(Mutex::new(None));

    let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> = match target {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let cred_bytes =
                secrets_repo::get_secret(db, secrets, HUB_NODE_ID, "webdav", secret_name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;

            let base_url = base_url.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            let uploader_failure_for_task = uploader_failure.clone();

            tokio::spawn(async move {
                match targets::webdav::store_run_parts_rolling(
                    &base_url,
                    credentials,
                    &job_id,
                    &run_id,
                    rx,
                )
                .await
                {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        let source = Arc::new(error);
                        let snapshot = RollingUploaderFailureSnapshot::from_source(source.clone());
                        let mut guard = uploader_failure_for_task
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        *guard = Some(snapshot);
                        Err(anyhow::Error::new(RollingUploaderTaskError { source }))
                    }
                }
            })
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            let uploader_failure_for_task = uploader_failure.clone();

            tokio::task::spawn_blocking(move || {
                match targets::local_dir::store_run_parts_rolling(
                    Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    rx,
                ) {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        let source = Arc::new(error);
                        let snapshot = RollingUploaderFailureSnapshot::from_source(source.clone());
                        let mut guard = uploader_failure_for_task
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        *guard = Some(snapshot);
                        Err(anyhow::Error::new(RollingUploaderTaskError { source }))
                    }
                }
            })
        }
    };

    let on_part_finished: Box<dyn Fn(backup::LocalArtifact) -> std::io::Result<()> + Send> = {
        let uploader_failure = uploader_failure.clone();
        Box::new(move |part| {
            let part_name = part.name.clone();
            let part_size_bytes = part.size;

            tx.blocking_send(part).map_err(|_| {
                let snapshot = {
                    let guard = uploader_failure
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    guard.clone()
                };

                let dropped = RollingUploaderDroppedError {
                    part_name,
                    part_size_bytes,
                    uploader_message: snapshot.as_ref().map(|s| s.message.clone()),
                    uploader_error_kind: snapshot.as_ref().and_then(|s| s.error_kind.clone()),
                    uploader_error_code: snapshot.as_ref().and_then(|s| s.error_code.clone()),
                    hint: snapshot.as_ref().and_then(|s| s.hint.clone()),
                    http_status: snapshot.as_ref().and_then(|s| s.http_status),
                    attempt: snapshot.as_ref().and_then(|s| s.attempt),
                    max_attempts: snapshot.as_ref().and_then(|s| s.max_attempts),
                    source: snapshot.map(|s| s.source),
                };

                std::io::Error::new(std::io::ErrorKind::BrokenPipe, dropped)
            })?;
            Ok(())
        })
    };

    Ok((Some(on_part_finished), Some(handle)))
}

#[cfg(test)]
mod tests {
    use super::{RollingUploaderDroppedError, merge_packaging_and_uploader_errors};

    #[test]
    fn merge_prefers_uploader_error_when_packaging_only_reports_dropped_sender() {
        let packaging_error = anyhow::Error::new(RollingUploaderDroppedError {
            part_name: "payload.part000001".to_string(),
            part_size_bytes: 16 * 1024 * 1024,
            uploader_message: Some("uploader stopped".to_string()),
            uploader_error_kind: Some("network".to_string()),
            uploader_error_code: Some("webdav_network".to_string()),
            hint: Some("check transport".to_string()),
            http_status: None,
            attempt: Some(2),
            max_attempts: Some(3),
            source: None,
        });
        let uploader_error = anyhow::anyhow!("webdav put failed: HTTP 413");

        let merged = merge_packaging_and_uploader_errors(packaging_error, uploader_error);
        let text = merged.to_string();
        assert!(text.contains("packaging aborted after rolling uploader failure"));
        assert!(
            merged
                .chain()
                .any(|cause| cause.to_string().contains("HTTP 413"))
        );
    }
}
