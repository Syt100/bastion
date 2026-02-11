use std::collections::BTreeMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::types::{
    FilesystemSource, JobSpecV1, NotificationsV1, PipelineV1, RetentionPolicyV1, SqliteSource,
    TargetV1, VaultwardenSource,
};

pub const JOB_SPEC_VERSION_V2: u32 = 2;

pub const SOURCE_KIND_FILESYSTEM: &str = "filesystem";
pub const SOURCE_KIND_SQLITE: &str = "sqlite";
pub const SOURCE_KIND_VAULTWARDEN: &str = "vaultwarden";

pub const TARGET_KIND_WEBDAV: &str = "webdav";
pub const TARGET_KIND_LOCAL_DIR: &str = "local_dir";

pub const AUTH_REF_WEBDAV_CREDENTIALS: &str = "webdav_credentials";

fn default_driver_version() -> u32 {
    1
}

fn default_target_part_size_bytes() -> u64 {
    256 * 1024 * 1024
}

fn default_job_spec_v2_version() -> u32 {
    JOB_SPEC_VERSION_V2
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceEnvelopeV2 {
    #[serde(rename = "type")]
    pub driver_type: String,
    #[serde(default = "default_driver_version")]
    pub version: u32,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AuthRefV2 {
    pub secret_type: String,
    pub secret_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetEnvelopeV2 {
    #[serde(rename = "type")]
    pub driver_type: String,
    #[serde(default = "default_driver_version")]
    pub version: u32,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub auth_refs: BTreeMap<String, AuthRefV2>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobSpecV2 {
    #[serde(default = "default_job_spec_v2_version")]
    pub v: u32,
    #[serde(default)]
    pub pipeline: PipelineV1,
    #[serde(default)]
    pub notifications: NotificationsV1,
    #[serde(default)]
    pub retention: RetentionPolicyV1,
    pub source: SourceEnvelopeV2,
    pub target: TargetEnvelopeV2,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavTargetConfigV2 {
    base_url: String,
    #[serde(default = "default_target_part_size_bytes")]
    part_size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalDirTargetConfigV2 {
    base_dir: String,
    #[serde(default = "default_target_part_size_bytes")]
    part_size_bytes: u64,
}

pub fn parse_canonical_value(spec: &serde_json::Value) -> Result<JobSpecV2, anyhow::Error> {
    if looks_like_legacy_v1(spec) {
        let legacy: JobSpecV1 = serde_json::from_value(spec.clone())
            .context("failed to parse legacy job spec payload")?;
        return Ok(translate_v1_to_v2(&legacy)?);
    }

    match serde_json::from_value::<JobSpecV2>(spec.clone()) {
        Ok(v2) => Ok(v2),
        Err(v2_error) => {
            let legacy: JobSpecV1 = serde_json::from_value(spec.clone())
                .context("failed to parse legacy job spec payload")?;
            translate_v1_to_v2(&legacy)
                .with_context(|| format!("failed to parse canonical job spec payload: {v2_error}"))
        }
    }
}

fn looks_like_legacy_v1(spec: &serde_json::Value) -> bool {
    spec.as_object()
        .and_then(|obj| obj.get("type"))
        .and_then(serde_json::Value::as_str)
        .is_some()
}

pub fn translate_v1_to_v2(spec: &JobSpecV1) -> Result<JobSpecV2, anyhow::Error> {
    match spec {
        JobSpecV1::Filesystem {
            pipeline,
            notifications,
            retention,
            source,
            target,
            ..
        } => Ok(JobSpecV2 {
            v: JOB_SPEC_VERSION_V2,
            pipeline: pipeline.clone(),
            notifications: notifications.clone(),
            retention: retention.clone(),
            source: SourceEnvelopeV2 {
                driver_type: SOURCE_KIND_FILESYSTEM.to_string(),
                version: 1,
                config: serde_json::to_value(source)
                    .context("failed to encode filesystem source config")?,
            },
            target: translate_target_v1_to_v2(target)?,
        }),
        JobSpecV1::Sqlite {
            pipeline,
            notifications,
            retention,
            source,
            target,
            ..
        } => Ok(JobSpecV2 {
            v: JOB_SPEC_VERSION_V2,
            pipeline: pipeline.clone(),
            notifications: notifications.clone(),
            retention: retention.clone(),
            source: SourceEnvelopeV2 {
                driver_type: SOURCE_KIND_SQLITE.to_string(),
                version: 1,
                config: serde_json::to_value(source)
                    .context("failed to encode sqlite source config")?,
            },
            target: translate_target_v1_to_v2(target)?,
        }),
        JobSpecV1::Vaultwarden {
            pipeline,
            notifications,
            retention,
            source,
            target,
            ..
        } => Ok(JobSpecV2 {
            v: JOB_SPEC_VERSION_V2,
            pipeline: pipeline.clone(),
            notifications: notifications.clone(),
            retention: retention.clone(),
            source: SourceEnvelopeV2 {
                driver_type: SOURCE_KIND_VAULTWARDEN.to_string(),
                version: 1,
                config: serde_json::to_value(source)
                    .context("failed to encode vaultwarden source config")?,
            },
            target: translate_target_v1_to_v2(target)?,
        }),
    }
}

fn translate_target_v1_to_v2(target: &TargetV1) -> Result<TargetEnvelopeV2, anyhow::Error> {
    match target {
        TargetV1::Webdav {
            base_url,
            secret_name,
            part_size_bytes,
        } => {
            let mut auth_refs = BTreeMap::new();
            auth_refs.insert(
                AUTH_REF_WEBDAV_CREDENTIALS.to_string(),
                AuthRefV2 {
                    secret_type: "webdav".to_string(),
                    secret_name: secret_name.clone(),
                },
            );

            Ok(TargetEnvelopeV2 {
                driver_type: TARGET_KIND_WEBDAV.to_string(),
                version: 1,
                config: serde_json::json!({
                    "base_url": base_url,
                    "part_size_bytes": part_size_bytes,
                }),
                auth_refs,
            })
        }
        TargetV1::LocalDir {
            base_dir,
            part_size_bytes,
        } => Ok(TargetEnvelopeV2 {
            driver_type: TARGET_KIND_LOCAL_DIR.to_string(),
            version: 1,
            config: serde_json::json!({
                "base_dir": base_dir,
                "part_size_bytes": part_size_bytes,
            }),
            auth_refs: BTreeMap::new(),
        }),
    }
}

pub fn translate_v2_to_v1(spec: &JobSpecV2) -> Result<JobSpecV1, anyhow::Error> {
    if spec.v != JOB_SPEC_VERSION_V2 {
        anyhow::bail!("unsupported canonical job spec version");
    }

    let source = translate_source_v2_to_v1(&spec.source)?;
    let target = translate_target_v2_to_v1(&spec.target)?;

    match source {
        SourceConfigV1::Filesystem(source) => Ok(JobSpecV1::Filesystem {
            v: 1,
            pipeline: spec.pipeline.clone(),
            notifications: spec.notifications.clone(),
            retention: spec.retention.clone(),
            source,
            target,
        }),
        SourceConfigV1::Sqlite(source) => Ok(JobSpecV1::Sqlite {
            v: 1,
            pipeline: spec.pipeline.clone(),
            notifications: spec.notifications.clone(),
            retention: spec.retention.clone(),
            source,
            target,
        }),
        SourceConfigV1::Vaultwarden(source) => Ok(JobSpecV1::Vaultwarden {
            v: 1,
            pipeline: spec.pipeline.clone(),
            notifications: spec.notifications.clone(),
            retention: spec.retention.clone(),
            source,
            target,
        }),
    }
}

enum SourceConfigV1 {
    Filesystem(FilesystemSource),
    Sqlite(SqliteSource),
    Vaultwarden(VaultwardenSource),
}

fn translate_source_v2_to_v1(source: &SourceEnvelopeV2) -> Result<SourceConfigV1, anyhow::Error> {
    if source.driver_type.trim().is_empty() {
        anyhow::bail!("source.type is required");
    }
    if source.version == 0 {
        anyhow::bail!("source.version must be >= 1");
    }

    match (source.driver_type.as_str(), source.version) {
        (SOURCE_KIND_FILESYSTEM, 1) => {
            let config = serde_json::from_value::<FilesystemSource>(source.config.clone())
                .context("invalid filesystem source config")?;
            Ok(SourceConfigV1::Filesystem(config))
        }
        (SOURCE_KIND_SQLITE, 1) => {
            let config = serde_json::from_value::<SqliteSource>(source.config.clone())
                .context("invalid sqlite source config")?;
            Ok(SourceConfigV1::Sqlite(config))
        }
        (SOURCE_KIND_VAULTWARDEN, 1) => {
            let config = serde_json::from_value::<VaultwardenSource>(source.config.clone())
                .context("invalid vaultwarden source config")?;
            Ok(SourceConfigV1::Vaultwarden(config))
        }
        (kind, version) => anyhow::bail!("unsupported source driver: {kind}@{version}"),
    }
}

fn translate_target_v2_to_v1(target: &TargetEnvelopeV2) -> Result<TargetV1, anyhow::Error> {
    if target.driver_type.trim().is_empty() {
        anyhow::bail!("target.type is required");
    }
    if target.version == 0 {
        anyhow::bail!("target.version must be >= 1");
    }

    match (target.driver_type.as_str(), target.version) {
        (TARGET_KIND_WEBDAV, 1) => translate_webdav_target_v2_to_v1(target),
        (TARGET_KIND_LOCAL_DIR, 1) => translate_local_dir_target_v2_to_v1(target),
        (kind, version) => anyhow::bail!("unsupported target driver: {kind}@{version}"),
    }
}

fn translate_webdav_target_v2_to_v1(target: &TargetEnvelopeV2) -> Result<TargetV1, anyhow::Error> {
    reject_inline_target_credentials(&target.config)?;

    let config: WebdavTargetConfigV2 =
        serde_json::from_value(target.config.clone()).context("invalid webdav target config")?;

    let secret_name = resolve_secret_name_from_auth_refs(target, "webdav")?;

    Ok(TargetV1::Webdav {
        base_url: config.base_url,
        secret_name,
        part_size_bytes: config.part_size_bytes,
    })
}

fn reject_inline_target_credentials(config: &serde_json::Value) -> Result<(), anyhow::Error> {
    let Some(obj) = config.as_object() else {
        anyhow::bail!("target.config must be an object");
    };

    for key in ["username", "password", "token", "secret_name"] {
        if obj.contains_key(key) {
            anyhow::bail!(
                "target.config.{key} is not allowed; credentials must use target.auth_refs"
            );
        }
    }

    Ok(())
}

fn resolve_secret_name_from_auth_refs(
    target: &TargetEnvelopeV2,
    expected_secret_type: &str,
) -> Result<String, anyhow::Error> {
    let preferred = target.auth_refs.get(AUTH_REF_WEBDAV_CREDENTIALS);

    let secret = if let Some(preferred) = preferred {
        preferred
    } else {
        let mut matches = target.auth_refs.values().filter(|candidate| {
            candidate.secret_type.trim() == expected_secret_type
                && !candidate.secret_name.trim().is_empty()
        });

        let first = matches.next().ok_or_else(|| {
            anyhow::anyhow!("target.auth_refs must include {expected_secret_type} credentials")
        })?;
        if matches.next().is_some() {
            anyhow::bail!(
                "target.auth_refs has multiple {expected_secret_type} credentials; only one is supported"
            );
        }
        first
    };

    if secret.secret_type.trim() != expected_secret_type {
        anyhow::bail!("target.auth_refs secret_type must be {expected_secret_type}");
    }
    let secret_name = secret.secret_name.trim();
    if secret_name.is_empty() {
        anyhow::bail!("target.auth_refs secret_name is required");
    }

    Ok(secret_name.to_string())
}

fn translate_local_dir_target_v2_to_v1(
    target: &TargetEnvelopeV2,
) -> Result<TargetV1, anyhow::Error> {
    if !target.auth_refs.is_empty() {
        anyhow::bail!("target.auth_refs is not supported for local_dir target");
    }

    let config: LocalDirTargetConfigV2 =
        serde_json::from_value(target.config.clone()).context("invalid local_dir target config")?;

    Ok(TargetV1::LocalDir {
        base_dir: config.base_dir,
        part_size_bytes: config.part_size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_webdav_translation_preserves_secret_ref_as_auth_ref() {
        let spec = JobSpecV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            notifications: Default::default(),
            retention: Default::default(),
            source: FilesystemSource {
                pre_scan: true,
                paths: vec!["/tmp".to_string()],
                root: "".to_string(),
                include: vec![],
                exclude: vec![],
                symlink_policy: Default::default(),
                hardlink_policy: Default::default(),
                error_policy: Default::default(),
                snapshot_mode: Default::default(),
                snapshot_provider: None,
                consistency_policy: Default::default(),
                consistency_fail_threshold: None,
                upload_on_consistency_failure: None,
            },
            target: TargetV1::Webdav {
                base_url: "https://example.invalid/backup".to_string(),
                secret_name: "main".to_string(),
                part_size_bytes: 1024 * 1024,
            },
        };

        let v2 = translate_v1_to_v2(&spec).expect("translate");
        assert_eq!(v2.v, JOB_SPEC_VERSION_V2);
        assert_eq!(v2.target.driver_type, TARGET_KIND_WEBDAV);
        assert_eq!(
            v2.target.auth_refs.get(AUTH_REF_WEBDAV_CREDENTIALS),
            Some(&AuthRefV2 {
                secret_type: "webdav".to_string(),
                secret_name: "main".to_string(),
            })
        );
    }

    #[test]
    fn v2_rejects_inline_webdav_credentials() {
        let payload = serde_json::json!({
            "v": 2,
            "pipeline": {},
            "source": {
                "type": "filesystem",
                "version": 1,
                "config": {
                    "paths": ["/tmp"]
                }
            },
            "target": {
                "type": "webdav",
                "version": 1,
                "config": {
                    "base_url": "https://example.invalid/backup",
                    "username": "u",
                    "password": "p"
                },
                "auth_refs": {
                    "webdav_credentials": {
                        "secret_type": "webdav",
                        "secret_name": "main"
                    }
                }
            }
        });

        let canonical = parse_canonical_value(&payload).expect("parse");
        let err = translate_v2_to_v1(&canonical).expect_err("must reject inline creds");
        assert!(
            err.to_string()
                .contains("credentials must use target.auth_refs"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn v2_to_v1_accepts_webdav_auth_ref() {
        let payload = serde_json::json!({
            "v": 2,
            "pipeline": {},
            "source": {
                "type": "sqlite",
                "version": 1,
                "config": {
                    "path": "/tmp/db.sqlite3"
                }
            },
            "target": {
                "type": "webdav",
                "version": 1,
                "config": {
                    "base_url": "https://example.invalid/backup",
                    "part_size_bytes": 4096
                },
                "auth_refs": {
                    "webdav_credentials": {
                        "secret_type": "webdav",
                        "secret_name": "main"
                    }
                }
            }
        });

        let canonical = parse_canonical_value(&payload).expect("parse");
        let translated = translate_v2_to_v1(&canonical).expect("translate");
        let JobSpecV1::Sqlite { target, .. } = translated else {
            panic!("expected sqlite spec");
        };
        let TargetV1::Webdav {
            secret_name,
            part_size_bytes,
            ..
        } = target
        else {
            panic!("expected webdav target");
        };

        assert_eq!(secret_name, "main");
        assert_eq!(part_size_bytes, 4096);
    }

    #[test]
    fn parse_canonical_value_accepts_legacy_payloads() {
        let legacy = serde_json::json!({
            "v": 1,
            "type": "vaultwarden",
            "pipeline": {},
            "source": { "data_dir": "/tmp/vw" },
            "target": { "type": "local_dir", "base_dir": "/tmp/out" }
        });

        let canonical = parse_canonical_value(&legacy).expect("parse");
        assert_eq!(canonical.v, JOB_SPEC_VERSION_V2);
        assert_eq!(canonical.source.driver_type, SOURCE_KIND_VAULTWARDEN);
        assert_eq!(canonical.target.driver_type, TARGET_KIND_LOCAL_DIR);
    }
}
