use std::path::PathBuf;

use bastion_core::HUB_NODE_ID;
use sqlx::SqlitePool;
use tracing::debug;
use url::Url;

use bastion_core::job_spec;
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

#[derive(Debug)]
pub(super) enum TargetAccess {
    Webdav {
        client: Box<WebdavClient>,
        run_url: Url,
    },
    LocalDir {
        run_dir: PathBuf,
    },
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

pub(super) async fn ensure_complete(access: &TargetAccess) -> Result<(), anyhow::Error> {
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
