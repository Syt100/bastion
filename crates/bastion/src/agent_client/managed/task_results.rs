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
