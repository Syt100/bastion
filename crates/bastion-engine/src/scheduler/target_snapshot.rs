use bastion_core::job_spec;
use bastion_driver_api::DriverId;
use bastion_driver_registry::builtins;

pub(super) fn build_run_target_snapshot(
    node_id: &str,
    spec: &job_spec::JobSpecV1,
) -> Result<serde_json::Value, anyhow::Error> {
    let (driver_id, target_config) = resolve_target_driver_input(spec)?;
    let target_snapshot =
        builtins::target_registry().snapshot_redacted(&driver_id, &target_config)?;

    Ok(serde_json::json!({
        "node_id": node_id,
        "target": target_snapshot,
    }))
}

fn resolve_target_driver_input(
    spec: &job_spec::JobSpecV1,
) -> Result<(DriverId, serde_json::Value), anyhow::Error> {
    let target = extract_target(spec);

    match target {
        job_spec::TargetV1::LocalDir { base_dir, .. } => Ok((
            builtins::local_dir_driver_id(),
            serde_json::json!({ "base_dir": base_dir }),
        )),
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => Ok((
            builtins::webdav_driver_id(),
            serde_json::json!({
                "base_url": base_url,
                "secret_name": secret_name,
            }),
        )),
    }
}

fn extract_target(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::build_run_target_snapshot;

    #[test]
    fn build_run_target_snapshot_local_dir_includes_node_and_base_dir() {
        let spec = bastion_core::job_spec::parse_value(&serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "paths": ["/"] },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }))
        .expect("parse spec");

        let snapshot = build_run_target_snapshot("node1", &spec).expect("snapshot");
        assert_eq!(snapshot["node_id"], "node1");
        assert_eq!(snapshot["target"]["type"], "local_dir");
        assert_eq!(snapshot["target"]["base_dir"], "/tmp");
    }

    #[test]
    fn build_run_target_snapshot_webdav_redacts_credentials_and_normalizes_slash() {
        let spec = bastion_core::job_spec::parse_value(&serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "paths": ["/"] },
            "target": {
                "type": "webdav",
                "base_url": "https://user:pass@example.com/base?q=1#frag",
                "secret_name": "primary"
            }
        }))
        .expect("parse spec");

        let snapshot = build_run_target_snapshot("node1", &spec).expect("snapshot");
        assert_eq!(snapshot["node_id"], "node1");
        assert_eq!(snapshot["target"]["type"], "webdav");
        assert_eq!(snapshot["target"]["secret_name"], "primary");

        let base_url = snapshot["target"]["base_url"].as_str().expect("base_url");
        let url = Url::parse(base_url).expect("parse redacted url");
        assert_eq!(url.username(), "");
        assert!(url.password().is_none());
        assert!(url.query().is_none());
        assert!(url.fragment().is_none());
        assert!(url.path().ends_with('/'));
    }

    #[test]
    fn build_run_target_snapshot_webdav_keeps_invalid_url_literal() {
        let spec = bastion_core::job_spec::parse_value(&serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "paths": ["/"] },
            "target": {
                "type": "webdav",
                "base_url": "not a url",
                "secret_name": "primary"
            }
        }))
        .expect("parse spec");

        let snapshot = build_run_target_snapshot("node1", &spec).expect("snapshot");
        assert_eq!(snapshot["target"]["base_url"], "not a url");
    }
}
