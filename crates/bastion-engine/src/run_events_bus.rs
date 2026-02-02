use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use time::OffsetDateTime;
use tokio::sync::broadcast;

use bastion_storage::runs_repo::RunEvent;

const DEFAULT_CAPACITY: usize = 1024;
const DEFAULT_IDLE_TTL_SECONDS: i64 = 10 * 60;
const DEFAULT_PRUNE_EVERY_OPS: u64 = 128;

#[derive(Debug)]
struct BusEntry {
    tx: broadcast::Sender<RunEvent>,
    last_used_at: i64,
}

#[derive(Debug)]
pub struct RunEventsBus {
    capacity: usize,
    idle_ttl_seconds: i64,
    prune_every_ops: u64,
    ops: AtomicU64,
    inner: Mutex<HashMap<String, BusEntry>>,
}

impl Default for RunEventsBus {
    fn default() -> Self {
        Self::new()
    }
}

impl RunEventsBus {
    pub fn new() -> Self {
        Self::new_with_options(
            DEFAULT_CAPACITY,
            DEFAULT_IDLE_TTL_SECONDS,
            DEFAULT_PRUNE_EVERY_OPS,
        )
    }

    pub fn new_with_options(capacity: usize, idle_ttl_seconds: i64, prune_every_ops: u64) -> Self {
        Self {
            capacity: capacity.max(1),
            idle_ttl_seconds: idle_ttl_seconds.max(0),
            prune_every_ops: prune_every_ops.max(1),
            ops: AtomicU64::new(0),
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn subscribe(&self, run_id: &str) -> broadcast::Receiver<RunEvent> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        self.maybe_prune_locked(&mut inner, now);

        let entry = inner.entry(run_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.capacity);
            BusEntry {
                tx,
                last_used_at: now,
            }
        });
        entry.last_used_at = now;
        entry.tx.subscribe()
    }

    pub fn publish(&self, event: &RunEvent) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        self.maybe_prune_locked(&mut inner, now);

        let entry = inner.entry(event.run_id.clone()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.capacity);
            BusEntry {
                tx,
                last_used_at: now,
            }
        });

        entry.last_used_at = now;
        let _ = entry.tx.send(event.clone());
    }

    fn maybe_prune_locked(&self, inner: &mut HashMap<String, BusEntry>, now: i64) {
        let ops = self.ops.fetch_add(1, Ordering::Relaxed) + 1;
        if !ops.is_multiple_of(self.prune_every_ops) {
            return;
        }

        inner.retain(|_, entry| {
            if entry.tx.receiver_count() > 0 {
                return true;
            }
            let idle_seconds = now.saturating_sub(entry.last_used_at);
            idle_seconds <= self.idle_ttl_seconds
        });
    }
}

#[cfg(test)]
mod tests {
    use super::RunEvent;
    use super::RunEventsBus;

    #[tokio::test]
    async fn publish_delivers_to_subscribers_after_mutex_poisoning() {
        let bus = RunEventsBus::new_with_options(8, 60, 1);

        // Simulate a panic while holding the lock, which poisons the mutex.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = bus.inner.lock().unwrap();
            panic!("poison");
        }));

        let mut rx = bus.subscribe("run1");

        bus.publish(&RunEvent {
            run_id: "run1".to_string(),
            seq: 1,
            ts: 100,
            level: "info".to_string(),
            kind: "test".to_string(),
            message: "hello".to_string(),
            fields: None,
        });

        let got = rx.recv().await.expect("recv");
        assert_eq!(got.seq, 1);
        assert_eq!(got.message, "hello");
    }

    #[tokio::test]
    async fn publish_delivers_to_subscribers() {
        let bus = RunEventsBus::new_with_options(8, 60, 1);
        let mut rx = bus.subscribe("run1");

        bus.publish(&RunEvent {
            run_id: "run1".to_string(),
            seq: 1,
            ts: 100,
            level: "info".to_string(),
            kind: "test".to_string(),
            message: "hello".to_string(),
            fields: None,
        });

        let got = rx.recv().await.expect("recv");
        assert_eq!(got.seq, 1);
        assert_eq!(got.message, "hello");
    }
}
