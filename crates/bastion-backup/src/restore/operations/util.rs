use std::path::{Path, PathBuf};

use bastion_core::manifest::ManifestV1;
use sqlx::SqlitePool;

use bastion_storage::secrets::SecretsCrypto;

use super::super::unpack::PayloadDecryption;

pub(in crate::restore::operations) async fn resolve_payload_decryption(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    manifest: &ManifestV1,
) -> Result<PayloadDecryption, anyhow::Error> {
    match manifest.pipeline.encryption.as_str() {
        "none" => Ok(PayloadDecryption::None),
        "age" => {
            let key_name = manifest
                .pipeline
                .encryption_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("missing manifest.pipeline.encryption_key"))?;

            let identity = crate::backup_encryption::get_age_identity(db, secrets, key_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing backup age identity: {}", key_name))?;
            Ok(PayloadDecryption::AgeX25519 { identity })
        }
        other => anyhow::bail!("unsupported manifest.pipeline.encryption: {}", other),
    }
}

pub(in crate::restore::operations) fn operation_dir(data_dir: &Path, op_id: &str) -> PathBuf {
    data_dir.join("operations").join(op_id)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use tempfile::TempDir;
    use uuid::Uuid;

    use bastion_core::manifest::{ArtifactFormatV1, EntryIndexRef, ManifestV1, PipelineSettings};
    use bastion_storage::{db, secrets::SecretsCrypto};

    use super::resolve_payload_decryption;
    use crate::backup_encryption::ensure_age_identity;
    use crate::restore::PayloadDecryption;

    fn manifest_with_encryption(encryption: &str, encryption_key: Option<&str>) -> ManifestV1 {
        ManifestV1 {
            format_version: ManifestV1::FORMAT_VERSION,
            job_id: Uuid::nil(),
            run_id: Uuid::nil(),
            started_at: "2026-02-02T00:00:00Z".to_string(),
            ended_at: "2026-02-02T00:00:01Z".to_string(),
            pipeline: PipelineSettings {
                format: ArtifactFormatV1::ArchiveV1,
                tar: "pax".to_string(),
                compression: "zstd".to_string(),
                encryption: encryption.to_string(),
                encryption_key: encryption_key.map(|v| v.to_string()),
                split_bytes: 1,
            },
            artifacts: Vec::new(),
            entry_index: EntryIndexRef {
                name: "entries.jsonl.zst".to_string(),
                count: 0,
            },
        }
    }

    #[tokio::test]
    async fn resolve_payload_decryption_none_returns_none() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let manifest = manifest_with_encryption("none", None);
        let dec = resolve_payload_decryption(&pool, &crypto, &manifest)
            .await
            .unwrap();
        assert!(matches!(dec, PayloadDecryption::None));
    }

    #[tokio::test]
    async fn resolve_payload_decryption_age_requires_key_name() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let manifest = manifest_with_encryption("age", None);
        let err = resolve_payload_decryption(&pool, &crypto, &manifest)
            .await
            .expect_err("expected error");
        assert!(format!("{err:#}").contains("missing manifest.pipeline.encryption_key"));
    }

    #[tokio::test]
    async fn resolve_payload_decryption_age_requires_stored_identity() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let manifest = manifest_with_encryption("age", Some("primary"));
        let err = resolve_payload_decryption(&pool, &crypto, &manifest)
            .await
            .expect_err("expected error");
        assert!(format!("{err:#}").contains("missing backup age identity"));
    }

    #[tokio::test]
    async fn resolve_payload_decryption_age_returns_identity() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let identity = ensure_age_identity(&pool, &crypto, "primary")
            .await
            .unwrap();

        let manifest = manifest_with_encryption("age", Some("primary"));
        let dec = resolve_payload_decryption(&pool, &crypto, &manifest)
            .await
            .unwrap();
        match dec {
            PayloadDecryption::AgeX25519 { identity: got } => {
                assert_eq!(got.trim(), identity.trim());
                age::x25519::Identity::from_str(got.trim()).unwrap();
            }
            other => panic!("unexpected decryption variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn resolve_payload_decryption_rejects_unknown_encryption() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let manifest = manifest_with_encryption("weird", None);
        assert!(
            resolve_payload_decryption(&pool, &crypto, &manifest)
                .await
                .is_err()
        );
    }
}
