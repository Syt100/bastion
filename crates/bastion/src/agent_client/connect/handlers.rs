use std::path::Path;
use std::sync::Arc;

use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, BackupAgeIdentitySecretV1, BackupRunTaskV1, JobConfigV1,
    OperationResultV1, PROTOCOL_VERSION, RestoreTaskV1, WebdavSecretV1,
};
use bastion_core::run_failure::RunFailedWithSummary;

use super::super::identity::AgentIdentityV1;
use super::super::managed::{
    load_cached_operation_result, load_cached_task_result, load_managed_webdav_credentials,
    save_managed_config_snapshot, save_managed_secrets_snapshot, save_task_result,
};
use super::super::util::is_ws_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HandlerFlow {
    Continue,
    Reconnect,
}

async fn send_json<S>(tx: &mut S, msg: &impl serde::Serialize) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let text = serde_json::to_string(msg)?;
    if tx.send(Message::Text(text.into())).await.is_err() {
        return Ok(HandlerFlow::Reconnect);
    }
    Ok(HandlerFlow::Continue)
}

pub(super) async fn handle_secrets_snapshot(
    identity: &AgentIdentityV1,
    data_dir: &Path,
    node_id: String,
    issued_at: i64,
    webdav: Vec<WebdavSecretV1>,
    backup_age_identities: Vec<BackupAgeIdentitySecretV1>,
) -> HandlerFlow {
    if node_id != identity.agent_id {
        warn!(
            agent_id = %identity.agent_id,
            node_id = %node_id,
            "received secrets snapshot for unexpected node_id; ignoring"
        );
        return HandlerFlow::Continue;
    }

    if let Err(error) = save_managed_secrets_snapshot(
        data_dir,
        &node_id,
        issued_at,
        &webdav,
        &backup_age_identities,
    ) {
        warn!(
            agent_id = %identity.agent_id,
            error = %error,
            "failed to persist secrets snapshot"
        );
    } else {
        debug!(
            agent_id = %identity.agent_id,
            webdav = webdav.len(),
            backup_age_identities = backup_age_identities.len(),
            "persisted secrets snapshot"
        );
    }

    HandlerFlow::Continue
}

pub(super) async fn handle_config_snapshot<S>(
    tx: &mut S,
    identity: &AgentIdentityV1,
    data_dir: &Path,
    node_id: String,
    snapshot_id: String,
    issued_at: i64,
    jobs: Vec<JobConfigV1>,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    if node_id != identity.agent_id {
        warn!(
            agent_id = %identity.agent_id,
            node_id = %node_id,
            "received config snapshot for unexpected node_id; ignoring"
        );
        return Ok(HandlerFlow::Continue);
    }

    if let Err(error) =
        save_managed_config_snapshot(data_dir, &node_id, &snapshot_id, issued_at, &jobs)
    {
        warn!(
            agent_id = %identity.agent_id,
            snapshot_id = %snapshot_id,
            error = %error,
            "failed to persist config snapshot"
        );
    } else {
        debug!(
            agent_id = %identity.agent_id,
            snapshot_id = %snapshot_id,
            jobs = jobs.len(),
            "persisted config snapshot"
        );
    }

    let ack = AgentToHubMessageV1::ConfigAck {
        v: PROTOCOL_VERSION,
        snapshot_id,
    };
    send_json(tx, &ack).await
}

pub(super) async fn handle_task<S>(
    tx: &mut S,
    data_dir: &Path,
    run_lock: Arc<tokio::sync::Mutex<()>>,
    task_id: String,
    task: Box<BackupRunTaskV1>,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let run_id = task.run_id.clone();
    debug!(task_id = %task_id, run_id = %run_id, "received task");

    if let Some(cached) = load_cached_task_result(data_dir, &task_id, &run_id) {
        debug!(
            task_id = %task_id,
            run_id = %run_id,
            "replaying cached task result"
        );
        let ack = AgentToHubMessageV1::Ack {
            v: PROTOCOL_VERSION,
            task_id: task_id.clone(),
        };
        if send_json(tx, &ack).await? == HandlerFlow::Reconnect {
            return Ok(HandlerFlow::Reconnect);
        }

        if send_json(tx, &cached).await? == HandlerFlow::Reconnect {
            return Ok(HandlerFlow::Reconnect);
        }
        return Ok(HandlerFlow::Continue);
    }

    let ack = AgentToHubMessageV1::Ack {
        v: PROTOCOL_VERSION,
        task_id: task_id.clone(),
    };
    if send_json(tx, &ack).await? == HandlerFlow::Reconnect {
        return Ok(HandlerFlow::Reconnect);
    }

    let _guard = run_lock.lock().await;
    match super::super::handle_backup_task(data_dir, tx, &task_id, *task).await {
        Ok(()) => {}
        Err(error) => {
            if is_ws_error(&error) {
                warn!(
                    task_id = %task_id,
                    run_id = %run_id,
                    error = %error,
                    "task aborted due to websocket error; reconnecting"
                );
                return Ok(HandlerFlow::Reconnect);
            }

            warn!(task_id = %task_id, run_id = %run_id, error = %error, "task failed");
            let summary = error
                .downcast_ref::<RunFailedWithSummary>()
                .map(|e| e.summary.clone());
            let result = AgentToHubMessageV1::TaskResult {
                v: PROTOCOL_VERSION,
                task_id: task_id.clone(),
                run_id,
                status: "failed".to_string(),
                summary,
                error: Some(format!("{error:#}")),
            };

            if let Err(error) = save_task_result(data_dir, &result) {
                warn!(task_id = %task_id, error = %error, "failed to persist task result");
            }

            if send_json(tx, &result).await? == HandlerFlow::Reconnect {
                return Ok(HandlerFlow::Reconnect);
            }
        }
    }

    Ok(HandlerFlow::Continue)
}

pub(super) async fn handle_restore_task<S>(
    tx: &mut S,
    data_dir: &Path,
    run_lock: Arc<tokio::sync::Mutex<()>>,
    hub_streams: &super::super::hub_stream::HubStreamManager,
    task_id: String,
    task: Box<RestoreTaskV1>,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let op_id = task.op_id.clone();
    let run_id = task.run_id.clone();
    debug!(task_id = %task_id, op_id = %op_id, run_id = %run_id, "received restore task");

    if let Some(cached) = load_cached_operation_result(data_dir, &op_id) {
        debug!(task_id = %task_id, op_id = %op_id, "replaying cached restore result");
        let ack = AgentToHubMessageV1::Ack {
            v: PROTOCOL_VERSION,
            task_id: task_id.clone(),
        };
        if send_json(tx, &ack).await? == HandlerFlow::Reconnect {
            return Ok(HandlerFlow::Reconnect);
        }
        if send_json(tx, &cached).await? == HandlerFlow::Reconnect {
            return Ok(HandlerFlow::Reconnect);
        }
        return Ok(HandlerFlow::Continue);
    }

    let ack = AgentToHubMessageV1::Ack {
        v: PROTOCOL_VERSION,
        task_id: task_id.clone(),
    };
    if send_json(tx, &ack).await? == HandlerFlow::Reconnect {
        return Ok(HandlerFlow::Reconnect);
    }

    let _guard = run_lock.lock().await;
    match super::super::handle_restore_task(data_dir, tx, hub_streams, &task_id, *task).await {
        Ok(()) => {}
        Err(error) => {
            if is_ws_error(&error) {
                warn!(
                    task_id = %task_id,
                    op_id = %op_id,
                    run_id = %run_id,
                    error = %error,
                    "restore task aborted due to websocket error; reconnecting"
                );
                return Ok(HandlerFlow::Reconnect);
            }

            warn!(
                task_id = %task_id,
                op_id = %op_id,
                run_id = %run_id,
                error = %error,
                "restore task failed"
            );

            let result = AgentToHubMessageV1::OperationResult {
                v: PROTOCOL_VERSION,
                result: OperationResultV1 {
                    op_id: op_id.clone(),
                    status: "failed".to_string(),
                    summary: None,
                    error: Some(format!("{error:#}")),
                },
            };

            if let Err(error) = save_task_result(data_dir, &result) {
                warn!(task_id = %task_id, error = %error, "failed to persist restore result");
            }

            if send_json(tx, &result).await? == HandlerFlow::Reconnect {
                return Ok(HandlerFlow::Reconnect);
            }
        }
    }

    Ok(HandlerFlow::Continue)
}

pub(super) struct FsListRequest {
    pub(super) request_id: String,
    pub(super) path: String,
    pub(super) cursor: Option<String>,
    pub(super) limit: Option<u32>,
    pub(super) q: Option<String>,
    pub(super) kind: Option<String>,
    pub(super) hide_dotfiles: Option<bool>,
    pub(super) type_sort: Option<String>,
    pub(super) sort_by: Option<String>,
    pub(super) sort_dir: Option<String>,
    pub(super) size_min_bytes: Option<u64>,
    pub(super) size_max_bytes: Option<u64>,
}

pub(super) struct WebdavListRequest {
    pub(super) request_id: String,
    pub(super) base_url: String,
    pub(super) secret_name: String,
    pub(super) path: String,
    pub(super) cursor: Option<String>,
    pub(super) limit: Option<u32>,
    pub(super) q: Option<String>,
    pub(super) kind: Option<String>,
    pub(super) hide_dotfiles: Option<bool>,
    pub(super) type_sort: Option<String>,
    pub(super) sort_by: Option<String>,
    pub(super) sort_dir: Option<String>,
    pub(super) size_min_bytes: Option<u64>,
    pub(super) size_max_bytes: Option<u64>,
}

pub(super) async fn handle_fs_list<S>(
    tx: &mut S,
    req: FsListRequest,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let FsListRequest {
        request_id,
        path,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        type_sort,
        sort_by,
        sort_dir,
        size_min_bytes,
        size_max_bytes,
    } = req;

    let path = path.trim().to_string();
    let cursor = cursor.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let q = q.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let kind = kind.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let type_sort = type_sort.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let sort_by = sort_by.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let sort_dir = sort_dir.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });

    let opts = super::super::fs_list::FsListOptions {
        cursor,
        limit: limit.map(|v| v.max(1)),
        q,
        kind,
        hide_dotfiles: hide_dotfiles.unwrap_or(false),
        type_sort,
        sort_by,
        sort_dir,
        size_min_bytes,
        size_max_bytes,
    };

    let result = tokio::task::spawn_blocking(move || {
        super::super::fs_list::fs_list_dir_entries_paged(&path, opts)
    })
    .await;

    let (entries, next_cursor, total, error) = match result {
        Ok(Ok(page)) => (page.entries, page.next_cursor, Some(page.total), None),
        Ok(Err(msg)) => (Vec::new(), None, None, Some(msg)),
        Err(error) => (
            Vec::new(),
            None,
            None,
            Some(format!("fs list task failed: {error}")),
        ),
    };

    let msg = AgentToHubMessageV1::FsListResult {
        v: PROTOCOL_VERSION,
        request_id,
        entries,
        next_cursor,
        total,
        error,
    };
    send_json(tx, &msg).await
}

pub(super) async fn handle_webdav_list<S>(
    tx: &mut S,
    data_dir: &Path,
    req: WebdavListRequest,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let WebdavListRequest {
        request_id,
        base_url,
        secret_name,
        path,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        type_sort,
        sort_by,
        sort_dir,
        size_min_bytes,
        size_max_bytes,
    } = req;

    let base_url = base_url.trim().to_string();
    let secret_name = secret_name.trim().to_string();
    let path = path.trim().to_string();
    let cursor = cursor.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let q = q.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let kind = kind.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let type_sort = type_sort.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let sort_by = sort_by.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let sort_dir = sort_dir.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });

    let (entries, next_cursor, total, error_code, error) = match load_managed_webdav_credentials(
        data_dir,
        &secret_name,
    )? {
        Some(credentials) => {
            let opts = super::super::webdav_list::WebdavListOptions {
                cursor,
                limit: limit.map(|v| v.max(1)),
                q,
                kind,
                hide_dotfiles: hide_dotfiles.unwrap_or(false),
                type_sort,
                sort_by,
                sort_dir,
                size_min_bytes,
                size_max_bytes,
            };

            match super::super::webdav_list::webdav_list_dir_entries_paged(
                &base_url,
                credentials,
                &path,
                opts,
            )
            .await
            {
                Ok(page) => (page.entries, page.next_cursor, Some(page.total), None, None),
                Err(e) => (
                    Vec::new(),
                    None,
                    None,
                    Some(e.code),
                    Some(e.message),
                ),
            }
        }
        None => (
            Vec::new(),
            None,
            None,
            Some("missing_webdav_secret".to_string()),
            Some("missing webdav secret for agent".to_string()),
        ),
    };

    let msg = AgentToHubMessageV1::WebdavListResult {
        v: PROTOCOL_VERSION,
        request_id,
        entries,
        next_cursor,
        total,
        error_code,
        error,
    };
    send_json(tx, &msg).await
}
