use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};
use url::Url;

use crate::config::AgentArgs;

const IDENTITY_FILE_NAME: &str = "agent.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AgentIdentityV1 {
    v: u32,
    hub_url: String,
    agent_id: String,
    agent_key: String,
    name: Option<String>,
    enrolled_at: i64,
}

#[derive(Debug, Serialize)]
struct EnrollRequest<'a> {
    token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct EnrollResponse {
    agent_id: String,
    agent_key: String,
}

#[derive(Debug, Serialize)]
struct HelloMessage<'a> {
    v: u32,
    #[serde(rename = "type")]
    msg_type: &'static str,
    agent_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    info: AgentInfo<'a>,
    capabilities: AgentCapabilities<'a>,
}

#[derive(Debug, Serialize)]
struct PingMessage {
    v: u32,
    #[serde(rename = "type")]
    msg_type: &'static str,
}

#[derive(Debug, Serialize)]
struct AgentInfo<'a> {
    version: &'a str,
    os: &'a str,
    arch: &'a str,
}

#[derive(Debug, Serialize)]
struct AgentCapabilities<'a> {
    backup: Vec<&'a str>,
}

pub async fn run(args: AgentArgs) -> Result<(), anyhow::Error> {
    if args.heartbeat_seconds == 0 {
        anyhow::bail!("heartbeat_seconds must be > 0");
    }

    let data_dir = crate::data_dir::resolve_data_dir(args.data_dir)?;
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
    let mut backoff = Duration::from_secs(1);

    loop {
        let action = connect_and_run(&ws_url, &identity, heartbeat).await;
        match action {
            Ok(LoopAction::Exit) => return Ok(()),
            Ok(LoopAction::Reconnect) => {
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
            Err(error) => {
                warn!(error = %error, "agent connection failed; retrying");
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopAction {
    Reconnect,
    Exit,
}

async fn connect_and_run(
    ws_url: &Url,
    identity: &AgentIdentityV1,
    heartbeat: Duration,
) -> Result<LoopAction, anyhow::Error> {
    let mut req = ws_url.as_str().into_client_request()?;
    req.headers_mut().insert(
        AUTHORIZATION,
        format!("Bearer {}", identity.agent_key).parse()?,
    );

    let (socket, _) = tokio_tungstenite::connect_async(req).await?;
    let (mut tx, mut rx) = socket.split();

    let hello = HelloMessage {
        v: 1,
        msg_type: "hello",
        agent_id: &identity.agent_id,
        name: identity.name.as_deref(),
        info: AgentInfo {
            version: env!("CARGO_PKG_VERSION"),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        },
        capabilities: AgentCapabilities {
            backup: vec!["filesystem", "sqlite", "vaultwarden"],
        },
    };
    tx.send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

    let mut tick = tokio::time::interval(heartbeat);
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                let _ = tx.send(Message::Close(None)).await;
                return Ok(LoopAction::Exit);
            }
            _ = tick.tick() => {
                let ping = PingMessage { v: 1, msg_type: "ping" };
                if tx.send(Message::Text(serde_json::to_string(&ping)?.into())).await.is_err() {
                    return Ok(LoopAction::Reconnect);
                }
            }
            msg = rx.next() => {
                let Some(msg) = msg else {
                    return Ok(LoopAction::Reconnect);
                };
                match msg {
                    Ok(Message::Text(_)) => {}
                    Ok(Message::Close(_)) => return Ok(LoopAction::Reconnect),
                    Ok(_) => {}
                    Err(_) => return Ok(LoopAction::Reconnect),
                }
            }
        }
    }
}

fn identity_path(data_dir: &Path) -> PathBuf {
    data_dir.join(IDENTITY_FILE_NAME)
}

fn normalize_base_url(raw: &str) -> Result<Url, anyhow::Error> {
    let mut url = Url::parse(raw)?;
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

fn agent_ws_url(base_url: &Url) -> Result<Url, anyhow::Error> {
    let mut url = base_url.clone();
    match url.scheme() {
        "http" => url.set_scheme("ws").ok(),
        "https" => url.set_scheme("wss").ok(),
        other => anyhow::bail!("unsupported hub_url scheme: {other}"),
    };
    Ok(url.join("agent/ws")?)
}

async fn enroll(base_url: &Url, token: &str, name: Option<&str>) -> Result<EnrollResponse, anyhow::Error> {
    let enroll_url = base_url.join("agent/enroll")?;
    let res = reqwest::Client::new()
        .post(enroll_url)
        .json(&EnrollRequest { token, name })
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("enroll failed: HTTP {status}: {text}");
    }

    Ok(res.json::<EnrollResponse>().await?)
}

fn load_identity(path: &Path) -> Result<Option<AgentIdentityV1>, anyhow::Error> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice::<AgentIdentityV1>(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn save_identity(path: &Path, identity: &AgentIdentityV1) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(identity)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{agent_ws_url, normalize_base_url, save_identity};

    #[test]
    fn normalize_base_url_appends_slash() {
        let url = normalize_base_url("http://localhost:9876").unwrap();
        assert_eq!(url.as_str(), "http://localhost:9876/");
    }

    #[test]
    fn agent_ws_url_converts_scheme() {
        let base = normalize_base_url("https://hub.example.com/bastion").unwrap();
        let ws = agent_ws_url(&base).unwrap();
        assert_eq!(ws.as_str(), "wss://hub.example.com/bastion/agent/ws");
    }

    #[test]
    fn identity_is_written_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("agent.json");
        let id = super::AgentIdentityV1 {
            v: 1,
            hub_url: "http://localhost:9876/".to_string(),
            agent_id: "a".to_string(),
            agent_key: "k".to_string(),
            name: Some("n".to_string()),
            enrolled_at: 1,
        };

        save_identity(&path, &id).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("\"agent_id\""));
    }
}
