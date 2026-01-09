use tracing::warn;

use bastion_core::job_spec;

use super::WorkerLoopCtx;

pub(super) async fn enqueue_for_run_spec(
    ctx: &WorkerLoopCtx<'_>,
    spec: &job_spec::JobSpecV1,
    run_id: &str,
) {
    match crate::notifications::enqueue_for_run_spec(ctx.db, spec, run_id).await {
        Ok(true) => ctx.notifications_notify.notify_one(),
        Ok(false) => {}
        Err(error) => {
            warn!(
                run_id = %run_id,
                error = %error,
                "failed to enqueue notifications"
            );
        }
    }
}
