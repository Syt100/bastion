use globset::Glob;
use url::Url;

use super::JOB_SPEC_VERSION;
use super::types::{
    EncryptionV1, FilesystemSource, JobSpecV1, NotificationsModeV1, NotificationsV1, PipelineV1,
    RetentionPolicyV1, TargetV1, VaultwardenSource, WebdavRawTreeDirectModeV1,
};
use crate::manifest::ArtifactFormatV1;

pub fn parse_value(spec: &serde_json::Value) -> Result<JobSpecV1, anyhow::Error> {
    Ok(serde_json::from_value(spec.clone())?)
}

pub fn validate_value(spec: &serde_json::Value) -> Result<(), anyhow::Error> {
    let spec = parse_value(spec)?;
    validate(&spec)
}

pub fn validate(spec: &JobSpecV1) -> Result<(), anyhow::Error> {
    match spec {
        JobSpecV1::Filesystem {
            v,
            pipeline,
            notifications,
            retention,
            source,
            target,
        } => {
            validate_version(*v)?;
            validate_pipeline(pipeline)?;
            validate_webdav_raw_tree_direct_filesystem(pipeline, source, target)?;
            validate_notifications(notifications)?;
            validate_retention(retention)?;
            validate_filesystem_source(source)?;
            validate_target(target)?;
        }
        JobSpecV1::Sqlite {
            v,
            pipeline,
            notifications,
            retention,
            source,
            target,
        } => {
            validate_version(*v)?;
            validate_pipeline(pipeline)?;
            validate_webdav_raw_tree_direct_unsupported("sqlite", pipeline)?;
            validate_notifications(notifications)?;
            validate_retention(retention)?;
            if source.path.trim().is_empty() {
                anyhow::bail!("sqlite.source.path is required");
            }
            validate_target(target)?;
        }
        JobSpecV1::Vaultwarden {
            v,
            pipeline,
            notifications,
            retention,
            source,
            target,
        } => {
            validate_version(*v)?;
            validate_pipeline(pipeline)?;
            validate_webdav_raw_tree_direct_unsupported("vaultwarden", pipeline)?;
            validate_notifications(notifications)?;
            validate_retention(retention)?;
            validate_vaultwarden_source(source)?;
            validate_target(target)?;
        }
    }

    Ok(())
}

fn validate_retention(retention: &RetentionPolicyV1) -> Result<(), anyhow::Error> {
    const MAX_KEEP_LAST: u32 = 10_000;
    const MAX_KEEP_DAYS: u32 = 3650; // 10 years
    const MAX_DELETE_PER_TICK: u32 = 10_000;
    const MAX_DELETE_PER_DAY: u32 = 100_000;

    if let Some(v) = retention.keep_last
        && v > MAX_KEEP_LAST
    {
        anyhow::bail!("retention.keep_last must be <= {MAX_KEEP_LAST}");
    }

    if let Some(v) = retention.keep_days
        && v > MAX_KEEP_DAYS
    {
        anyhow::bail!("retention.keep_days must be <= {MAX_KEEP_DAYS}");
    }

    if retention.max_delete_per_tick == 0 || retention.max_delete_per_tick > MAX_DELETE_PER_TICK {
        anyhow::bail!("retention.max_delete_per_tick must be within 1..={MAX_DELETE_PER_TICK}");
    }

    if retention.max_delete_per_day == 0 || retention.max_delete_per_day > MAX_DELETE_PER_DAY {
        anyhow::bail!("retention.max_delete_per_day must be within 1..={MAX_DELETE_PER_DAY}");
    }

    if retention.enabled {
        let keep_last = retention.keep_last.unwrap_or(0);
        let keep_days = retention.keep_days.unwrap_or(0);
        if keep_last == 0 && keep_days == 0 {
            anyhow::bail!(
                "retention.enabled is true but both retention.keep_last and retention.keep_days are empty"
            );
        }
    }

    Ok(())
}

fn validate_pipeline(pipeline: &PipelineV1) -> Result<(), anyhow::Error> {
    if pipeline.format == ArtifactFormatV1::RawTreeV1
        && !matches!(pipeline.encryption, EncryptionV1::None)
    {
        anyhow::bail!("pipeline.encryption is not supported when pipeline.format is raw_tree_v1");
    }

    match &pipeline.encryption {
        EncryptionV1::None => {}
        EncryptionV1::AgeX25519 { key_name } => {
            if key_name.trim().is_empty() {
                anyhow::bail!("pipeline.encryption.key_name is required");
            }
        }
    }
    Ok(())
}

fn validate_webdav_raw_tree_direct_unsupported(
    job_type: &'static str,
    pipeline: &PipelineV1,
) -> Result<(), anyhow::Error> {
    if pipeline.webdav.raw_tree_direct.mode != WebdavRawTreeDirectModeV1::Off {
        anyhow::bail!("pipeline.webdav.raw_tree_direct is not supported for {job_type} jobs");
    }
    Ok(())
}

fn validate_webdav_raw_tree_direct_filesystem(
    pipeline: &PipelineV1,
    source: &FilesystemSource,
    target: &TargetV1,
) -> Result<(), anyhow::Error> {
    let direct = &pipeline.webdav.raw_tree_direct;
    if direct.mode == WebdavRawTreeDirectModeV1::Off {
        return Ok(());
    }

    if pipeline.format != ArtifactFormatV1::RawTreeV1 {
        anyhow::bail!("pipeline.webdav.raw_tree_direct requires pipeline.format=raw_tree_v1");
    }

    if !matches!(target, TargetV1::Webdav { .. }) {
        anyhow::bail!("pipeline.webdav.raw_tree_direct requires target.type=webdav");
    }

    if source.consistency_policy == super::types::ConsistencyPolicyV1::Fail
        && !source.upload_on_consistency_failure.unwrap_or(false)
    {
        anyhow::bail!(
            "pipeline.webdav.raw_tree_direct is not allowed when filesystem.source.consistency_policy=fail and filesystem.source.upload_on_consistency_failure is false"
        );
    }

    if let Some(limits) = direct.limits.as_ref() {
        const MAX_CONCURRENCY: u32 = 128;
        const MAX_QPS: u32 = 10_000;
        const MAX_BURST: u32 = 100_000;

        if limits.concurrency == 0 || limits.concurrency > MAX_CONCURRENCY {
            anyhow::bail!(
                "pipeline.webdav.raw_tree_direct.limits.concurrency must be within 1..={MAX_CONCURRENCY}"
            );
        }

        for (name, qps) in [
            ("put_qps", limits.put_qps),
            ("head_qps", limits.head_qps),
            ("mkcol_qps", limits.mkcol_qps),
        ] {
            if let Some(qps) = qps
                && (qps == 0 || qps > MAX_QPS)
            {
                anyhow::bail!(
                    "pipeline.webdav.raw_tree_direct.limits.{name} must be within 1..={MAX_QPS}"
                );
            }
        }

        if let Some(burst) = limits.burst
            && (burst == 0 || burst > MAX_BURST)
        {
            anyhow::bail!(
                "pipeline.webdav.raw_tree_direct.limits.burst must be within 1..={MAX_BURST}"
            );
        }
    }

    Ok(())
}

fn validate_notifications(notifications: &NotificationsV1) -> Result<(), anyhow::Error> {
    if notifications.mode == NotificationsModeV1::Custom {
        for name in notifications
            .wecom_bot
            .iter()
            .chain(notifications.email.iter())
        {
            if name.trim().is_empty() {
                anyhow::bail!("notifications destination name is required");
            }
        }
    }
    Ok(())
}

fn validate_version(v: u32) -> Result<(), anyhow::Error> {
    if v != JOB_SPEC_VERSION {
        anyhow::bail!("unsupported job spec version");
    }
    Ok(())
}

fn validate_globs(patterns: &[String]) -> Result<(), anyhow::Error> {
    for p in patterns {
        let _ = Glob::new(p)?;
    }
    Ok(())
}

fn validate_filesystem_source(source: &FilesystemSource) -> Result<(), anyhow::Error> {
    let has_paths = source.paths.iter().any(|p| !p.trim().is_empty());
    let has_root = !source.root.trim().is_empty();
    if !has_paths && !has_root {
        anyhow::bail!("filesystem.source.paths (or legacy filesystem.source.root) is required");
    }
    validate_globs(&source.include)?;
    validate_globs(&source.exclude)?;
    Ok(())
}

fn validate_vaultwarden_source(source: &VaultwardenSource) -> Result<(), anyhow::Error> {
    if source.data_dir.trim().is_empty() {
        anyhow::bail!("vaultwarden.source.data_dir is required");
    }
    Ok(())
}

fn validate_target(target: &TargetV1) -> Result<(), anyhow::Error> {
    match target {
        TargetV1::Webdav {
            base_url,
            secret_name,
            part_size_bytes,
        } => {
            if base_url.trim().is_empty() {
                anyhow::bail!("target.base_url is required");
            }
            if secret_name.trim().is_empty() {
                anyhow::bail!("target.secret_name is required");
            }
            let url = Url::parse(base_url.trim())?;
            if !matches!(url.scheme(), "http" | "https") {
                anyhow::bail!("target.base_url must be http(s)");
            }
            if *part_size_bytes < 1024 * 1024 {
                anyhow::bail!("target.part_size_bytes must be >= 1048576");
            }
        }
        TargetV1::LocalDir {
            base_dir,
            part_size_bytes,
        } => {
            if base_dir.trim().is_empty() {
                anyhow::bail!("target.base_dir is required");
            }
            if *part_size_bytes < 1024 * 1024 {
                anyhow::bail!("target.part_size_bytes must be >= 1048576");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_value;

    #[test]
    fn webdav_raw_tree_direct_rejects_non_filesystem_jobs() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "sqlite",
          "pipeline": {
            "format": "raw_tree_v1",
            "webdav": { "raw_tree_direct": { "mode": "auto" } }
          },
          "source": { "path": "/tmp/db.sqlite3" },
          "target": { "type": "webdav", "base_url": "https://example.invalid/backup", "secret_name": "s" }
        });
        let err = validate_value(&spec).expect_err("invalid");
        assert!(
            err.to_string().contains("raw_tree_direct"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn webdav_raw_tree_direct_requires_webdav_target_and_raw_tree_format() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "pipeline": {
            "format": "archive_v1",
            "webdav": { "raw_tree_direct": { "mode": "on" } }
          },
          "source": { "paths": ["/"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" }
        });
        let err = validate_value(&spec).expect_err("invalid");
        assert!(
            err.to_string().contains("raw_tree_v1"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn webdav_raw_tree_direct_rejects_consistency_fail_without_upload_on_failure() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "pipeline": {
            "format": "raw_tree_v1",
            "webdav": { "raw_tree_direct": { "mode": "auto" } }
          },
          "source": {
            "paths": ["/"],
            "consistency_policy": "fail",
            "consistency_fail_threshold": 0,
            "upload_on_consistency_failure": false
          },
          "target": { "type": "webdav", "base_url": "https://example.invalid/backup", "secret_name": "s" }
        });
        let err = validate_value(&spec).expect_err("invalid");
        assert!(
            err.to_string().contains("upload_on_consistency_failure"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn webdav_raw_tree_direct_validates_limits() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "pipeline": {
            "format": "raw_tree_v1",
            "webdav": {
              "raw_tree_direct": {
                "mode": "on",
                "limits": { "concurrency": 0 }
              }
            }
          },
          "source": { "paths": ["/"] },
          "target": { "type": "webdav", "base_url": "https://example.invalid/backup", "secret_name": "s" }
        });
        let err = validate_value(&spec).expect_err("invalid");
        assert!(
            err.to_string().contains("limits.concurrency"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn retention_disabled_allows_empty_keep_rules() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" },
          "retention": { "enabled": false }
        });
        validate_value(&spec).expect("valid");
    }

    #[test]
    fn retention_enabled_requires_keep_last_or_keep_days() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" },
          "retention": { "enabled": true }
        });
        let err = validate_value(&spec).expect_err("invalid");
        assert!(
            err.to_string().contains("retention.enabled"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn retention_enabled_accepts_keep_last() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" },
          "retention": { "enabled": true, "keep_last": 3 }
        });
        validate_value(&spec).expect("valid");
    }

    #[test]
    fn retention_rejects_zero_safety_limits() {
        let spec = serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" },
          "retention": { "enabled": true, "keep_last": 1, "max_delete_per_tick": 0 }
        });
        let err = validate_value(&spec).expect_err("invalid");
        assert!(
            err.to_string().contains("retention.max_delete_per_tick"),
            "unexpected error: {err}"
        );
    }
}
