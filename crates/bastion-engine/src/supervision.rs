use tokio_util::sync::CancellationToken;

pub(crate) fn spawn_supervised<F>(
    name: &'static str,
    shutdown: CancellationToken,
    fut: F,
) -> tokio::task::JoinHandle<()>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        let handle = tokio::spawn(fut);
        match handle.await {
            Ok(()) => {
                if shutdown.is_cancelled() {
                    tracing::debug!(task = name, "background task stopped");
                } else {
                    tracing::error!(task = name, "background task exited unexpectedly");
                    shutdown.cancel();
                }
            }
            Err(error) => {
                if error.is_panic() {
                    tracing::error!(task = name, "background task panicked");
                } else {
                    tracing::error!(task = name, error = %error, "background task join failed");
                }
                shutdown.cancel();
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio_util::sync::CancellationToken;

    use super::spawn_supervised;

    #[tokio::test]
    async fn panic_cancels_shutdown() {
        let shutdown = CancellationToken::new();
        let handle = spawn_supervised("test_panic", shutdown.clone(), async move {
            panic!("boom");
        });

        tokio::time::timeout(Duration::from_secs(1), shutdown.cancelled())
            .await
            .expect("shutdown cancelled");
        handle.await.expect("supervisor join");
    }

    #[tokio::test]
    async fn graceful_stop_completes_when_shutdown_is_cancelled() {
        let shutdown = CancellationToken::new();
        let shutdown_task = shutdown.clone();
        let handle = spawn_supervised("test_graceful", shutdown.clone(), async move {
            shutdown_task.cancelled().await;
        });

        shutdown.cancel();
        handle.await.expect("supervisor join");
    }
}

