use std::sync::Arc;

use axum::extract::ConnectInfo;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::runs_repo;

use super::super::shared::{is_trusted_proxy, require_session};
use super::super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(in crate::http) struct RunEventsWsQuery {
    #[serde(default, alias = "after_seq")]
    after: Option<i64>,
}

pub(in crate::http) async fn run_events_ws(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    Query(query): Query<RunEventsWsQuery>,
    Path(run_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let _session = require_session(&state, &cookies).await?;
    require_ws_same_origin(&state, &headers, peer.ip())?;

    let run_exists = runs_repo::get_run(&state.db, &run_id).await?.is_some();
    if !run_exists {
        return Err(AppError::not_found("run_not_found", "Run not found"));
    }

    let after_seq = query.after.unwrap_or(0).max(0);
    let db = state.db.clone();
    let run_events_bus = state.run_events_bus.clone();
    Ok(ws.on_upgrade(move |socket| {
        handle_run_events_socket(db, run_id, after_seq, run_events_bus, socket)
    }))
}

fn require_ws_same_origin(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> Result<(), AppError> {
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("invalid_origin", "Invalid origin"))?;

    let expected_host = if is_trusted_proxy(state, peer_ip) {
        headers
            .get("x-forwarded-host")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                headers
                    .get(axum::http::header::HOST)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    } else {
        headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
    .ok_or_else(|| AppError::unauthorized("invalid_origin", "Invalid origin"))?;

    let expected_host = expected_host
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    let origin_host = match url::Url::parse(origin) {
        Ok(url) => url.host_str().unwrap_or("").to_ascii_lowercase(),
        Err(_) => return Err(AppError::unauthorized("invalid_origin", "Invalid origin")),
    };

    if origin_host != expected_host {
        return Err(AppError::unauthorized("invalid_origin", "Invalid origin"));
    }

    Ok(())
}

async fn handle_run_events_socket(
    db: SqlitePool,
    run_id: String,
    after_seq: i64,
    run_events_bus: Arc<RunEventsBus>,
    mut socket: WebSocket,
) {
    let mut last_seq = after_seq.max(0);
    let mut idle_after_end = 0u32;

    let mut rx = run_events_bus.subscribe(&run_id);

    // Catch up from SQLite after the requested sequence.
    loop {
        let events = match runs_repo::list_run_events_after_seq(&db, &run_id, last_seq, 200).await {
            Ok(v) => v,
            Err(_) => return,
        };
        if events.is_empty() {
            break;
        }
        idle_after_end = 0;
        for event in events {
            last_seq = last_seq.max(event.seq);
            let payload = match serde_json::to_string(&event) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if socket.send(Message::Text(payload.into())).await.is_err() {
                return;
            }
        }
    }

    let mut status_interval = tokio::time::interval(std::time::Duration::from_secs(3));
    status_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    status_interval.tick().await; // discard immediate tick

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        if event.seq <= last_seq {
                            continue;
                        }
                        idle_after_end = 0;
                        last_seq = event.seq;
                        let payload = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // The client fell behind; resync from SQLite after the last confirmed seq.
                        loop {
                            let events = match runs_repo::list_run_events_after_seq(&db, &run_id, last_seq, 200).await {
                                Ok(v) => v,
                                Err(_) => return,
                            };
                            if events.is_empty() {
                                break;
                            }
                            idle_after_end = 0;
                            for event in events {
                                last_seq = last_seq.max(event.seq);
                                let payload = match serde_json::to_string(&event) {
                                    Ok(s) => s,
                                    Err(_) => continue,
                                };
                                if socket.send(Message::Text(payload.into())).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = status_interval.tick() => {
                match runs_repo::get_run(&db, &run_id).await {
                    Ok(Some(run)) => {
                        let ended = !matches!(run.status, runs_repo::RunStatus::Queued | runs_repo::RunStatus::Running);
                        if ended {
                            idle_after_end += 1;
                            if idle_after_end >= 10 {
                                break;
                            }
                        } else {
                            idle_after_end = 0;
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
        }
    }
}
