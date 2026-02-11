use std::path::PathBuf;
use std::sync::Arc;

use bastion_core::HUB_NODE_ID;
use sqlx::SqlitePool;
use tracing::debug;

use bastion_core::job_spec;
use bastion_driver_api::{OpenReaderRequest, TargetRunReader};
use bastion_driver_registry::builtins;
use bastion_driver_registry::target_runtime::{self, WebdavRuntimeAuth};
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

pub(super) struct TargetAccess {
    node_id: String,
    reader: Arc<dyn TargetRunReader>,
}

impl std::fmt::Debug for TargetAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetAccess")
            .field("node_id", &self.node_id)
            .field("target_kind", &self.reader.target_kind())
            .field("location", &self.reader.describe_location())
            .finish()
    }
}

impl TargetAccess {
    pub(super) fn reader(&self) -> Arc<dyn TargetRunReader> {
        self.reader.clone()
    }

    pub(super) fn local_run_dir(&self) -> Option<PathBuf> {
        self.reader.local_run_dir()
    }
}

pub(super) struct ResolvedRunAccess {
    pub(super) run: runs_repo::Run,
    pub(super) access: TargetAccess,
}

fn target_ref(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

pub(super) async fn resolve_success_run_access(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_id: &str,
) -> Result<ResolvedRunAccess, anyhow::Error> {
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

    Ok(ResolvedRunAccess { run, access })
}

async fn resolve_target_config_for_reader(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    target: &job_spec::TargetV1,
) -> Result<(bastion_driver_api::DriverId, serde_json::Value), anyhow::Error> {
    let webdav_auth = match target {
        job_spec::TargetV1::Webdav { secret_name, .. } => {
            let secret_name = secret_name.trim();
            if secret_name.is_empty() {
                anyhow::bail!("webdav.secret_name is required");
            }

            let cred_bytes = secrets_repo::get_secret(db, secrets, node_id, "webdav", secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = bastion_targets::WebdavCredentials::from_json(&cred_bytes)?;
            Some(WebdavRuntimeAuth {
                username: credentials.username,
                password: credentials.password,
                secret_name: Some(secret_name.to_string()),
            })
        }
        job_spec::TargetV1::LocalDir { .. } => None,
    };

    target_runtime::runtime_input_for_job_target(target, webdav_auth.as_ref())
        .map_err(anyhow::Error::new)
}

async fn open_target_access(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    job_id: &str,
    run_id: &str,
    target: &job_spec::TargetV1,
) -> Result<TargetAccess, anyhow::Error> {
    let (driver_id, target_config) =
        resolve_target_config_for_reader(db, secrets, node_id, target).await?;

    let reader = builtins::target_registry().open_reader(
        &driver_id,
        OpenReaderRequest {
            job_id: job_id.to_string(),
            run_id: run_id.to_string(),
            target_config,
        },
    )?;

    debug!(
        job_id = %job_id,
        run_id = %run_id,
        node_id = %node_id,
        target = %reader.target_kind(),
        location = %reader.describe_location(),
        "resolved restore target access"
    );

    Ok(TargetAccess {
        node_id: node_id.to_string(),
        reader,
    })
}

pub(super) async fn ensure_complete(access: &TargetAccess) -> Result<(), anyhow::Error> {
    let exists = access.reader.complete_exists().await?;
    if !exists {
        anyhow::bail!("complete.json not found");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use bastion_storage::{db, jobs_repo, runs_repo, secrets::SecretsCrypto};

    use super::{TargetAccess, ensure_complete, resolve_success_run_access};

    #[tokio::test]
    async fn ensure_complete_local_dir_requires_complete_marker() {
        let tmp = TempDir::new().unwrap();
        let run_dir = tmp.path().join("job1").join("run1");
        std::fs::create_dir_all(&run_dir).unwrap();

        let reader = bastion_driver_registry::builtins::target_registry()
            .open_reader(
                &bastion_driver_registry::builtins::local_dir_driver_id(),
                bastion_driver_api::OpenReaderRequest {
                    job_id: "job1".to_string(),
                    run_id: "run1".to_string(),
                    target_config: serde_json::json!({
                        "base_dir": tmp.path().to_string_lossy().to_string(),
                    }),
                },
            )
            .expect("open reader");

        let access = TargetAccess {
            node_id: bastion_core::HUB_NODE_ID.to_string(),
            reader,
        };
        let err = ensure_complete(&access).await.unwrap_err();
        assert!(format!("{err:#}").contains("complete.json not found"));
    }

    #[tokio::test]
    async fn resolve_success_run_access_local_dir_happy_path() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let base_dir = tmp.path().join("artifacts");
        std::fs::create_dir_all(&base_dir).unwrap();

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            Some("UTC"),
            jobs_repo::OverlapPolicy::Queue,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "paths": ["/"] },
                "target": { "type": "local_dir", "base_dir": base_dir.to_string_lossy().to_string() }
            }),
        )
        .await
        .unwrap();

        let run = runs_repo::create_run(
            &pool,
            &job.id,
            runs_repo::RunStatus::Success,
            1,
            Some(2),
            None,
            None,
        )
        .await
        .unwrap();

        let run_dir = base_dir.join(&job.id).join(&run.id);
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::write(run_dir.join(crate::backup::COMPLETE_NAME), b"{}").unwrap();

        let resolved = resolve_success_run_access(&pool, &crypto, &run.id)
            .await
            .unwrap();

        assert_eq!(resolved.access.local_run_dir(), Some(run_dir));
        assert_eq!(resolved.run.id, run.id);
    }

    #[tokio::test]
    async fn resolve_success_run_access_rejects_non_success_runs() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let base_dir = tmp.path().join("artifacts");
        std::fs::create_dir_all(&base_dir).unwrap();

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            Some("UTC"),
            jobs_repo::OverlapPolicy::Queue,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "paths": ["/"] },
                "target": { "type": "local_dir", "base_dir": base_dir.to_string_lossy().to_string() }
            }),
        )
        .await
        .unwrap();

        let run = runs_repo::create_run(
            &pool,
            &job.id,
            runs_repo::RunStatus::Failed,
            1,
            Some(2),
            None,
            Some("boom"),
        )
        .await
        .unwrap();

        let err = resolve_success_run_access(&pool, &crypto, &run.id)
            .await
            .err()
            .expect("expected error");
        assert!(format!("{err:#}").contains("run is not successful"));
    }

    #[tokio::test]
    async fn resolve_success_run_access_webdav_requires_secret() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            Some("UTC"),
            jobs_repo::OverlapPolicy::Queue,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "paths": ["/"] },
                "target": {
                    "type": "webdav",
                    "base_url": "https://example.com/base/",
                    "secret_name": "primary"
                }
            }),
        )
        .await
        .unwrap();

        let run = runs_repo::create_run(
            &pool,
            &job.id,
            runs_repo::RunStatus::Success,
            1,
            Some(2),
            None,
            None,
        )
        .await
        .unwrap();

        let err = resolve_success_run_access(&pool, &crypto, &run.id)
            .await
            .err()
            .expect("expected error");
        assert!(format!("{err:#}").contains("missing webdav secret"));
    }
}
