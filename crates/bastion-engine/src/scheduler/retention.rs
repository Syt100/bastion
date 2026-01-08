use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use bastion_storage::runs_repo;

pub(super) async fn run_retention_loop(
    db: SqlitePool,
    run_retention_days: i64,
    shutdown: CancellationToken,
) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let cutoff = now.saturating_sub(run_retention_days.saturating_mul(24 * 60 * 60));

        match runs_repo::prune_runs_ended_before(&db, cutoff).await {
            Ok(pruned) => {
                if pruned > 0 {
                    info!(pruned, run_retention_days, "pruned old runs");
                }
            }
            Err(error) => {
                warn!(error = %error, "failed to prune old runs");
            }
        }

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 60)) => {}
        }
    }
}
