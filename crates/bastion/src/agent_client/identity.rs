use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

const IDENTITY_FILE_NAME: &str = "agent.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct AgentIdentityV1 {
    pub(super) v: u32,
    pub(super) hub_url: String,
    pub(super) agent_id: String,
    pub(super) agent_key: String,
    pub(super) name: Option<String>,
    pub(super) enrolled_at: i64,
}

#[derive(Debug, Serialize)]
struct EnrollRequest<'a> {
    token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub(super) struct EnrollResponse {
    pub(super) agent_id: String,
    pub(super) agent_key: String,
}

pub(super) fn identity_path(data_dir: &Path) -> PathBuf {
    data_dir.join(IDENTITY_FILE_NAME)
}

pub(super) async fn enroll(
    base_url: &Url,
    token: &str,
    name: Option<&str>,
) -> Result<EnrollResponse, anyhow::Error> {
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

pub(super) fn load_identity(path: &Path) -> Result<Option<AgentIdentityV1>, anyhow::Error> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice::<AgentIdentityV1>(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn save_identity(path: &Path, identity: &AgentIdentityV1) -> Result<(), anyhow::Error> {
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
    use super::save_identity;

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
