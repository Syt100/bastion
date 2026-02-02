use bastion_core::job_spec;
use url::Url;

pub(super) fn build_run_target_snapshot(
    node_id: &str,
    spec: &job_spec::JobSpecV1,
) -> serde_json::Value {
    match extract_target(spec) {
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            serde_json::json!({ "node_id": node_id, "target": { "type": "local_dir", "base_dir": base_dir } })
        }
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let base_url = redact_base_url(base_url);
            serde_json::json!({ "node_id": node_id, "target": { "type": "webdav", "base_url": base_url, "secret_name": secret_name } })
        }
    }
}

fn extract_target(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

fn redact_base_url(base_url: &str) -> String {
    let Ok(mut url) = Url::parse(base_url) else {
        return base_url.to_string();
    };

    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);

    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }

    url.to_string()
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
        .unwrap();

        let snapshot = build_run_target_snapshot("node1", &spec);
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
        .unwrap();

        let snapshot = build_run_target_snapshot("node1", &spec);
        assert_eq!(snapshot["node_id"], "node1");
        assert_eq!(snapshot["target"]["type"], "webdav");
        assert_eq!(snapshot["target"]["secret_name"], "primary");

        let base_url = snapshot["target"]["base_url"].as_str().unwrap();
        let url = Url::parse(base_url).unwrap();
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
        .unwrap();

        let snapshot = build_run_target_snapshot("node1", &spec);
        assert_eq!(snapshot["target"]["base_url"], "not a url");
    }
}
