use bastion_core::agent_protocol::TargetResolvedV1;
use bastion_core::job_spec;
use bastion_driver_api::{DriverError, DriverId};

use crate::builtins;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebdavRuntimeAuth {
    pub username: String,
    pub password: String,
    pub secret_name: Option<String>,
}

pub fn driver_id_for_job_target(target: &job_spec::TargetV1) -> DriverId {
    match target {
        job_spec::TargetV1::Webdav { .. } => builtins::webdav_driver_id(),
        job_spec::TargetV1::LocalDir { .. } => builtins::local_dir_driver_id(),
    }
}

pub fn runtime_input_for_job_target(
    target: &job_spec::TargetV1,
    webdav_auth: Option<&WebdavRuntimeAuth>,
) -> Result<(DriverId, serde_json::Value), DriverError> {
    match target {
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = require_non_empty(base_dir, "local_dir.base_dir")?;
            Ok((
                builtins::local_dir_driver_id(),
                serde_json::json!({ "base_dir": base_dir }),
            ))
        }
        job_spec::TargetV1::Webdav { base_url, .. } => {
            let base_url = require_non_empty(base_url, "webdav.base_url")?;
            let auth = webdav_auth.ok_or_else(|| {
                DriverError::auth("webdav credentials are required for runtime target config")
            })?;
            let username = require_non_empty(&auth.username, "webdav.username")?;
            let password = require_non_empty(&auth.password, "webdav.password")?;

            let mut out = serde_json::json!({
                "base_url": base_url,
                "username": username,
                "password": password,
            });
            if let Some(secret_name) = auth.secret_name.as_deref() {
                let secret_name = require_non_empty(secret_name, "webdav.secret_name")?;
                if let Some(obj) = out.as_object_mut() {
                    obj.insert(
                        "secret_name".to_string(),
                        serde_json::Value::String(secret_name.to_string()),
                    );
                }
            }

            Ok((builtins::webdav_driver_id(), out))
        }
    }
}

pub fn snapshot_input_for_job_target(
    target: &job_spec::TargetV1,
) -> Result<(DriverId, serde_json::Value), DriverError> {
    match target {
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = require_non_empty(base_dir, "local_dir.base_dir")?;
            Ok((
                builtins::local_dir_driver_id(),
                serde_json::json!({ "base_dir": base_dir }),
            ))
        }
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let base_url = require_non_empty(base_url, "webdav.base_url")?;
            let secret_name = require_non_empty(secret_name, "webdav.secret_name")?;
            Ok((
                builtins::webdav_driver_id(),
                serde_json::json!({
                    "base_url": base_url,
                    "secret_name": secret_name,
                }),
            ))
        }
    }
}

pub fn runtime_input_for_resolved_target(
    target: &TargetResolvedV1,
) -> Result<(DriverId, serde_json::Value), DriverError> {
    match target {
        TargetResolvedV1::LocalDir { base_dir, .. } => {
            let base_dir = require_non_empty(base_dir, "local_dir.base_dir")?;
            Ok((
                builtins::local_dir_driver_id(),
                serde_json::json!({ "base_dir": base_dir }),
            ))
        }
        TargetResolvedV1::Webdav {
            base_url,
            username,
            password,
            ..
        } => {
            let base_url = require_non_empty(base_url, "webdav.base_url")?;
            let username = require_non_empty(username, "webdav.username")?;
            let password = require_non_empty(password, "webdav.password")?;
            Ok((
                builtins::webdav_driver_id(),
                serde_json::json!({
                    "base_url": base_url,
                    "username": username,
                    "password": password,
                }),
            ))
        }
    }
}

fn require_non_empty<'a>(value: &'a str, field: &str) -> Result<&'a str, DriverError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DriverError::config(format!("{field} is required")));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::{
        WebdavRuntimeAuth, driver_id_for_job_target, runtime_input_for_job_target,
        runtime_input_for_resolved_target, snapshot_input_for_job_target,
    };

    use bastion_core::agent_protocol::TargetResolvedV1;
    use bastion_core::job_spec;

    #[test]
    fn driver_id_for_job_target_uses_builtin_ids() {
        let webdav = job_spec::TargetV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            secret_name: "primary".to_string(),
            part_size_bytes: 1,
        };
        let local = job_spec::TargetV1::LocalDir {
            base_dir: "/tmp".to_string(),
            part_size_bytes: 1,
        };

        assert_eq!(driver_id_for_job_target(&webdav).kind, "webdav");
        assert_eq!(driver_id_for_job_target(&local).kind, "local_dir");
    }

    #[test]
    fn runtime_input_for_job_target_requires_webdav_auth() {
        let target = job_spec::TargetV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            secret_name: "primary".to_string(),
            part_size_bytes: 1,
        };

        let err = runtime_input_for_job_target(&target, None).expect_err("must fail");
        assert_eq!(err.kind, bastion_driver_api::DriverErrorKind::Auth);
    }

    #[test]
    fn runtime_input_for_job_target_webdav_includes_secret_name() {
        let target = job_spec::TargetV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            secret_name: "primary".to_string(),
            part_size_bytes: 1,
        };

        let (_id, cfg) = runtime_input_for_job_target(
            &target,
            Some(&WebdavRuntimeAuth {
                username: "u".to_string(),
                password: "p".to_string(),
                secret_name: Some("primary".to_string()),
            }),
        )
        .expect("runtime input");

        assert_eq!(cfg["base_url"], "https://example.com/base/");
        assert_eq!(cfg["username"], "u");
        assert_eq!(cfg["password"], "p");
        assert_eq!(cfg["secret_name"], "primary");
    }

    #[test]
    fn runtime_input_for_resolved_target_builds_webdav_config() {
        let target = TargetResolvedV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            part_size_bytes: 1,
        };

        let (_id, cfg) = runtime_input_for_resolved_target(&target).expect("resolved input");
        assert_eq!(cfg["base_url"], "https://example.com/base/");
        assert_eq!(cfg["username"], "u");
        assert_eq!(cfg["password"], "p");
    }

    #[test]
    fn snapshot_input_for_job_target_requires_webdav_secret_name() {
        let target = job_spec::TargetV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            secret_name: " ".to_string(),
            part_size_bytes: 1,
        };

        let err = snapshot_input_for_job_target(&target).expect_err("must fail");
        assert_eq!(err.kind, bastion_driver_api::DriverErrorKind::Config);
    }
}
