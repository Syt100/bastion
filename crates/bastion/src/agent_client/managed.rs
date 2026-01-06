use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, JobConfigV1, PROTOCOL_VERSION, WebdavSecretV1,
};
use bastion_storage::secrets::{EncryptedSecret, SecretsCrypto};

use super::{
    MANAGED_CONFIG_FILE_NAME, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, MANAGED_SECRETS_FILE_NAME,
};

fn is_safe_task_id(task_id: &str) -> bool {
    !task_id.is_empty()
        && task_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn task_results_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent").join("task_results")
}

fn managed_secrets_path(data_dir: &Path) -> PathBuf {
    data_dir
        .join("agent")
        .join("managed")
        .join(MANAGED_SECRETS_FILE_NAME)
}

fn managed_config_path(data_dir: &Path) -> PathBuf {
    data_dir
        .join("agent")
        .join("managed")
        .join(MANAGED_CONFIG_FILE_NAME)
}

fn task_result_path(data_dir: &Path, task_id: &str) -> Option<PathBuf> {
    if !is_safe_task_id(task_id) {
        return None;
    }
    Some(task_results_dir(data_dir).join(format!("{task_id}.json")))
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedSecretsFileV1 {
    v: u32,
    node_id: String,
    issued_at: i64,
    saved_at: i64,
    #[serde(default)]
    webdav: Vec<ManagedWebdavSecretV1>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedWebdavSecretV1 {
    name: String,
    updated_at: i64,
    kid: u32,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

pub(super) fn save_managed_secrets_snapshot(
    data_dir: &Path,
    node_id: &str,
    issued_at: i64,
    webdav: &[WebdavSecretV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut entries = Vec::with_capacity(webdav.len());
    for secret in webdav {
        let payload = WebdavSecretPayload {
            username: secret.username.clone(),
            password: secret.password.clone(),
        };
        let bytes = serde_json::to_vec(&payload)?;
        let encrypted = crypto.encrypt(node_id, "webdav", &secret.name, &bytes)?;
        entries.push(ManagedWebdavSecretV1 {
            name: secret.name.clone(),
            updated_at: secret.updated_at,
            kid: encrypted.kid,
            nonce: encrypted.nonce.to_vec(),
            ciphertext: encrypted.ciphertext,
        });
    }

    let doc = ManagedSecretsFileV1 {
        v: 1,
        node_id: node_id.to_string(),
        issued_at,
        saved_at,
        webdav: entries,
    };

    let path = managed_secrets_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(&doc)?;
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

#[derive(Debug, Serialize, Deserialize)]
struct ManagedConfigFileV1 {
    v: u32,
    node_id: String,
    snapshot_id: String,
    issued_at: i64,
    saved_at: i64,
    encrypted: EncryptedBlobV1,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedBlobV1 {
    kid: u32,
    nonce_b64: String,
    ciphertext_b64: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ManagedConfigPlainV1 {
    pub(super) v: u32,
    pub(super) snapshot_id: String,
    pub(super) issued_at: i64,
    pub(super) jobs: Vec<JobConfigV1>,
}

pub(super) fn save_managed_config_snapshot(
    data_dir: &Path,
    node_id: &str,
    snapshot_id: &str,
    issued_at: i64,
    jobs: &[JobConfigV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let plain = ManagedConfigPlainV1 {
        v: 1,
        snapshot_id: snapshot_id.to_string(),
        issued_at,
        jobs: jobs.to_vec(),
    };
    let bytes = serde_json::to_vec(&plain)?;

    let encrypted = crypto.encrypt(node_id, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, &bytes)?;
    let encrypted = EncryptedBlobV1 {
        kid: encrypted.kid,
        nonce_b64: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(encrypted.nonce),
        ciphertext_b64: base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(encrypted.ciphertext),
    };

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let doc = ManagedConfigFileV1 {
        v: 1,
        node_id: node_id.to_string(),
        snapshot_id: snapshot_id.to_string(),
        issued_at,
        saved_at,
        encrypted,
    };

    let path = managed_config_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(&doc)?;
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

pub(super) fn load_managed_config_snapshot(
    data_dir: &Path,
    node_id: &str,
) -> Result<Option<ManagedConfigPlainV1>, anyhow::Error> {
    let path = managed_config_path(data_dir);
    let bytes = match std::fs::read(&path) {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let doc: ManagedConfigFileV1 = serde_json::from_slice(&bytes)?;
    if doc.v != 1 {
        anyhow::bail!("unsupported managed config snapshot version: {}", doc.v);
    }

    let nonce = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(doc.encrypted.nonce_b64.as_bytes())?;
    let ciphertext = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(doc.encrypted.ciphertext_b64.as_bytes())?;

    let nonce: [u8; 24] = nonce
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid nonce length"))?;

    let crypto = SecretsCrypto::load_or_create(data_dir)?;
    let secret = EncryptedSecret {
        kid: doc.encrypted.kid,
        nonce,
        ciphertext,
    };
    let plaintext = crypto.decrypt(node_id, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, &secret)?;
    let plain: ManagedConfigPlainV1 = serde_json::from_slice(&plaintext)?;
    Ok(Some(plain))
}

pub(super) fn load_cached_task_result(
    data_dir: &Path,
    task_id: &str,
    run_id: &str,
) -> Option<AgentToHubMessageV1> {
    let path = task_result_path(data_dir, task_id)?;
    let bytes = std::fs::read(path).ok()?;
    let msg = serde_json::from_slice::<AgentToHubMessageV1>(&bytes).ok()?;
    match &msg {
        AgentToHubMessageV1::TaskResult {
            v,
            task_id: saved_task_id,
            run_id: saved_run_id,
            ..
        } if *v == PROTOCOL_VERSION && saved_task_id == task_id && saved_run_id == run_id => {
            Some(msg)
        }
        _ => None,
    }
}

pub(super) fn save_task_result(
    data_dir: &Path,
    msg: &AgentToHubMessageV1,
) -> Result<(), anyhow::Error> {
    let AgentToHubMessageV1::TaskResult { task_id, .. } = msg else {
        return Ok(());
    };

    let Some(path) = task_result_path(data_dir, task_id) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(msg)?;
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
    use super::{
        ManagedConfigFileV1, ManagedSecretsFileV1, managed_config_path, managed_secrets_path,
    };

    #[test]
    fn managed_secrets_snapshot_is_persisted_encrypted() {
        let tmp = tempfile::tempdir().unwrap();
        let webdav = vec![bastion_core::agent_protocol::WebdavSecretV1 {
            name: "primary".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            updated_at: 10,
        }];

        super::save_managed_secrets_snapshot(tmp.path(), "a", 123, &webdav).unwrap();

        let path = managed_secrets_path(tmp.path());
        assert!(path.exists());

        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("pass"));

        let saved: ManagedSecretsFileV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(saved.v, 1);
        assert_eq!(saved.node_id, "a");
        assert_eq!(saved.issued_at, 123);
        assert_eq!(saved.webdav.len(), 1);
        assert_eq!(saved.webdav[0].name, "primary");
        assert_eq!(saved.webdav[0].updated_at, 10);
        assert_eq!(saved.webdav[0].nonce.len(), 24);
        assert!(!saved.webdav[0].ciphertext.is_empty());

        assert!(tmp.path().join("master.key").exists());
    }

    #[test]
    fn managed_config_snapshot_is_persisted_encrypted() {
        let tmp = tempfile::tempdir().unwrap();
        let jobs = vec![bastion_core::agent_protocol::JobConfigV1 {
            job_id: "job1".to_string(),
            name: "job1".to_string(),
            schedule: Some("0 */6 * * *".to_string()),
            overlap_policy: bastion_core::agent_protocol::OverlapPolicyV1::Queue,
            updated_at: 10,
            spec: bastion_core::agent_protocol::JobSpecResolvedV1::Filesystem {
                v: 1,
                pipeline: Default::default(),
                source: bastion_core::job_spec::FilesystemSource {
                    root: "/".to_string(),
                    include: vec![],
                    exclude: vec![],
                    symlink_policy: Default::default(),
                    hardlink_policy: Default::default(),
                    error_policy: bastion_core::job_spec::FsErrorPolicy::FailFast,
                },
                target: bastion_core::agent_protocol::TargetResolvedV1::LocalDir {
                    base_dir: "/tmp".to_string(),
                    part_size_bytes: 1024,
                },
            },
        }];

        super::save_managed_config_snapshot(tmp.path(), "a", "snap1", 123, &jobs).unwrap();

        let path = managed_config_path(tmp.path());
        assert!(path.exists());

        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        // Ensure the on-disk doc doesn't contain obvious plaintext fields.
        assert!(!text.contains("\"base_dir\""));
        assert!(!text.contains("0 */6 * * *"));

        let saved: ManagedConfigFileV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(saved.v, 1);
        assert_eq!(saved.node_id, "a");
        assert_eq!(saved.snapshot_id, "snap1");
        assert_eq!(saved.issued_at, 123);
        assert!(!saved.encrypted.nonce_b64.is_empty());
        assert!(!saved.encrypted.ciphertext_b64.is_empty());

        assert!(tmp.path().join("master.key").exists());
    }
}
