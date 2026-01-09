use tracing::warn;

use bastion_storage::runs_repo;

use super::WorkerLoopCtx;

pub(super) async fn claim_next_queued_run_or_wait(
    ctx: &WorkerLoopCtx<'_>,
) -> Option<runs_repo::Run> {
    let run = match runs_repo::claim_next_queued_run(ctx.db).await {
        Ok(v) => v,
        Err(error) => {
            warn!(error = %error, "failed to claim queued run");
            tokio::select! {
                _ = ctx.shutdown.cancelled() => {}
                _ = ctx.run_queue_notify.notified() => {}
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
            }
            return None;
        }
    };

    let Some(run) = run else {
        tokio::select! {
            _ = ctx.shutdown.cancelled() => {}
            _ = ctx.run_queue_notify.notified() => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
        }
        return None;
    };

    Some(run)
}
