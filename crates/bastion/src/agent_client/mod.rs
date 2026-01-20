use std::time::Duration;

use tracing::{info, warn};

use crate::config::AgentArgs;

mod connect;
mod fs_list;
mod hub_stream;
mod identity;
mod managed;
mod offline;
mod restore_task;
mod targets;
mod tasks;
mod util;
mod webdav_list;

use connect::{LoopAction, connect_and_run};
use identity::{AgentIdentityV1, enroll, identity_path, load_identity, save_identity};
use util::{agent_ws_url, jittered_backoff, normalize_base_url};

const MANAGED_SECRETS_FILE_NAME: &str = "secrets.json";
const MANAGED_CONFIG_FILE_NAME: &str = "config.json";
const MANAGED_CONFIG_KIND: &str = "agent_config_snapshot";
const MANAGED_CONFIG_NAME: &str = "config";

use restore_task::handle_restore_task;
use tasks::handle_backup_task;

pub async fn run(args: AgentArgs) -> Result<(), anyhow::Error> {
    if args.heartbeat_seconds == 0 {
        anyhow::bail!("heartbeat_seconds must be > 0");
    }

    let data_dir = bastion_config::data_dir::resolve_data_dir(args.data_dir)?;
    let base_url = normalize_base_url(&args.hub_url)?;

    let identity_path = identity_path(&data_dir);
    let identity = match load_identity(&identity_path)? {
        Some(v) => {
            let stored_url = normalize_base_url(&v.hub_url)?;
            if stored_url != base_url {
                anyhow::bail!(
                    "agent is already enrolled for hub_url={}, delete {} to re-enroll",
                    stored_url,
                    identity_path.display()
                );
            }
            v
        }
        None => {
            let Some(token) = args.enroll_token.as_deref() else {
                anyhow::bail!(
                    "agent is not enrolled yet; provide --enroll-token or set BASTION_AGENT_ENROLL_TOKEN"
                );
            };

            info!(hub_url = %base_url, "enrolling agent");
            let resp = enroll(&base_url, token, args.name.as_deref()).await?;
            let now = time::OffsetDateTime::now_utc().unix_timestamp();
            let identity = AgentIdentityV1 {
                v: 1,
                hub_url: base_url.to_string(),
                agent_id: resp.agent_id,
                agent_key: resp.agent_key,
                name: args.name.clone(),
                enrolled_at: now,
            };
            save_identity(&identity_path, &identity)?;
            identity
        }
    };

    let ws_url = agent_ws_url(&base_url)?;
    let heartbeat = Duration::from_secs(args.heartbeat_seconds);
    let pong_timeout = Duration::from_secs(args.heartbeat_seconds.saturating_mul(3));
    let mut backoff = Duration::from_secs(1);
    let mut attempt = 0u32;

    let run_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
    let (connected_tx, connected_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(offline::offline_scheduler_loop(
        data_dir.clone(),
        identity.agent_id.clone(),
        run_lock.clone(),
        connected_rx,
    ));

    loop {
        let action = connect_and_run(
            &ws_url,
            &identity,
            &data_dir,
            heartbeat,
            pong_timeout,
            run_lock.clone(),
            &connected_tx,
        )
        .await;
        match action {
            Ok(LoopAction::Exit) => return Ok(()),
            Ok(LoopAction::Reconnect) => {
                attempt = attempt.saturating_add(1);
                tokio::time::sleep(jittered_backoff(backoff, &identity.agent_id, attempt)).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
            Err(error) => {
                warn!(error = %error, "agent connection failed; retrying");
                attempt = attempt.saturating_add(1);
                tokio::time::sleep(jittered_backoff(backoff, &identity.agent_id, attempt)).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
        }
    }
}
