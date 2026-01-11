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
