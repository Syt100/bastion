use super::config_snapshot::ManagedConfigFileV1;
use super::paths::{managed_config_path, managed_secrets_path};
use super::secrets_snapshot::ManagedSecretsFileV1;

#[test]
fn managed_secrets_snapshot_is_persisted_encrypted() {
    let tmp = tempfile::tempdir().unwrap();
    let webdav = vec![bastion_core::agent_protocol::WebdavSecretV1 {
        name: "primary".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        updated_at: 10,
    }];
    let backup_age_identities = vec![bastion_core::agent_protocol::BackupAgeIdentitySecretV1 {
        name: "key1".to_string(),
        identity: "AGE-SECRET-KEY-1".to_string(),
        updated_at: 11,
    }];

    super::save_managed_secrets_snapshot(tmp.path(), "a", 123, &webdav, &backup_age_identities)
        .unwrap();

    let path = managed_secrets_path(tmp.path());
    assert!(path.exists());

    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(!text.contains("pass"));
    assert!(!text.contains("AGE-SECRET-KEY-1"));

    let saved: ManagedSecretsFileV1 = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved.v, 1);
    assert_eq!(saved.node_id, "a");
    assert_eq!(saved.issued_at, 123);
    assert_eq!(saved.webdav.len(), 1);
    assert_eq!(saved.webdav[0].name, "primary");
    assert_eq!(saved.webdav[0].updated_at, 10);
    assert_eq!(saved.webdav[0].nonce.len(), 24);
    assert!(!saved.webdav[0].ciphertext.is_empty());

    assert_eq!(saved.backup_age_identities.len(), 1);
    assert_eq!(saved.backup_age_identities[0].name, "key1");
    assert_eq!(saved.backup_age_identities[0].updated_at, 11);
    assert_eq!(saved.backup_age_identities[0].nonce.len(), 24);
    assert!(!saved.backup_age_identities[0].ciphertext.is_empty());

    let loaded = super::load_managed_backup_age_identity(tmp.path(), "key1")
        .unwrap()
        .unwrap();
    assert_eq!(loaded, "AGE-SECRET-KEY-1");

    assert!(tmp.path().join("master.key").exists());
}

#[test]
fn managed_config_snapshot_is_persisted_encrypted() {
    let tmp = tempfile::tempdir().unwrap();
    let jobs = vec![bastion_core::agent_protocol::JobConfigV1 {
        job_id: "job1".to_string(),
        name: "job1".to_string(),
        schedule: Some("0 */6 * * *".to_string()),
        schedule_timezone: Some("UTC".to_string()),
        overlap_policy: bastion_core::agent_protocol::OverlapPolicyV1::Queue,
        updated_at: 10,
        spec: bastion_core::agent_protocol::JobSpecResolvedV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            source: bastion_core::job_spec::FilesystemSource {
                pre_scan: true,
                paths: vec![],
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
