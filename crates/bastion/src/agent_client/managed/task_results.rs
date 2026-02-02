use std::path::Path;

use bastion_core::agent_protocol::{AgentToHubMessageV1, PROTOCOL_VERSION};

use super::io::write_json_pretty_atomic;
use super::paths::task_result_path;

pub(in super::super) fn load_cached_task_result(
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

pub(in super::super) fn load_cached_operation_result(
    data_dir: &Path,
    op_id: &str,
) -> Option<AgentToHubMessageV1> {
    let path = task_result_path(data_dir, op_id)?;
    let bytes = std::fs::read(path).ok()?;
    let msg = serde_json::from_slice::<AgentToHubMessageV1>(&bytes).ok()?;
    match &msg {
        AgentToHubMessageV1::OperationResult { v, result }
            if *v == PROTOCOL_VERSION && result.op_id == op_id =>
        {
            Some(msg)
        }
        _ => None,
    }
}

pub(in super::super) fn save_task_result(
    data_dir: &Path,
    msg: &AgentToHubMessageV1,
) -> Result<(), anyhow::Error> {
    let task_id = match msg {
        AgentToHubMessageV1::TaskResult { task_id, .. } => task_id.as_str(),
        AgentToHubMessageV1::OperationResult { result, .. } => result.op_id.as_str(),
        _ => return Ok(()),
    };

    let Some(path) = task_result_path(data_dir, task_id) else {
        return Ok(());
    };

    write_json_pretty_atomic(&path, msg)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bastion_core::agent_protocol::{AgentToHubMessageV1, OperationResultV1, PROTOCOL_VERSION};

    use super::{load_cached_operation_result, load_cached_task_result, save_task_result};

    #[test]
    fn save_and_load_task_result_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();

        let msg = AgentToHubMessageV1::TaskResult {
            v: PROTOCOL_VERSION,
            task_id: "task1".to_string(),
            run_id: "run1".to_string(),
            status: "success".to_string(),
            summary: Some(serde_json::json!({"k": "v"})),
            error: None,
        };

        save_task_result(tmp.path(), &msg).unwrap();

        let loaded =
            load_cached_task_result(tmp.path(), "task1", "run1").expect("cached task result");
        match loaded {
            AgentToHubMessageV1::TaskResult {
                v,
                task_id,
                run_id,
                status,
                summary,
                error,
            } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(task_id, "task1");
                assert_eq!(run_id, "run1");
                assert_eq!(status, "success");
                assert_eq!(summary, Some(serde_json::json!({"k": "v"})));
                assert_eq!(error, None);
            }
            other => panic!("unexpected message: {other:?}"),
        }

        assert!(load_cached_task_result(tmp.path(), "task1", "other-run").is_none());
    }

    #[test]
    fn load_cached_task_result_rejects_mismatched_protocol_version() {
        let tmp = tempfile::tempdir().unwrap();

        let msg = AgentToHubMessageV1::TaskResult {
            v: PROTOCOL_VERSION + 1,
            task_id: "task1".to_string(),
            run_id: "run1".to_string(),
            status: "success".to_string(),
            summary: None,
            error: None,
        };

        save_task_result(tmp.path(), &msg).unwrap();

        assert!(load_cached_task_result(tmp.path(), "task1", "run1").is_none());
    }

    #[test]
    fn save_and_load_operation_result_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();

        let msg = AgentToHubMessageV1::OperationResult {
            v: PROTOCOL_VERSION,
            result: OperationResultV1 {
                op_id: "op1".to_string(),
                status: "success".to_string(),
                summary: Some(serde_json::json!({"k": "v"})),
                error: None,
            },
        };

        save_task_result(tmp.path(), &msg).unwrap();

        let loaded = load_cached_operation_result(tmp.path(), "op1").expect("cached op result");
        match loaded {
            AgentToHubMessageV1::OperationResult { v, result } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(result.op_id, "op1");
                assert_eq!(result.status, "success");
                assert_eq!(result.summary, Some(serde_json::json!({"k": "v"})));
                assert_eq!(result.error, None);
            }
            other => panic!("unexpected message: {other:?}"),
        }

        assert!(load_cached_operation_result(tmp.path(), "other-op").is_none());
    }

    #[test]
    fn save_task_result_ignores_non_result_messages() {
        let tmp = tempfile::tempdir().unwrap();

        let msg = AgentToHubMessageV1::Ack {
            v: PROTOCOL_VERSION,
            task_id: "task1".to_string(),
        };

        save_task_result(tmp.path(), &msg).unwrap();

        // Ack messages should not create a cache directory or files.
        assert!(!tmp.path().join("agent").join("task_results").exists());
    }

    #[test]
    fn save_task_result_ignores_unsafe_task_ids() {
        let tmp = tempfile::tempdir().unwrap();

        let msg = AgentToHubMessageV1::TaskResult {
            v: PROTOCOL_VERSION,
            task_id: "task.with.dot".to_string(),
            run_id: "run1".to_string(),
            status: "success".to_string(),
            summary: None,
            error: None,
        };

        save_task_result(tmp.path(), &msg).unwrap();
        assert!(!tmp.path().join("agent").join("task_results").exists());
    }
}
