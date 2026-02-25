use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

#[derive(Clone, Default)]
pub(super) struct TaskCancelRegistry {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    run_tokens: HashMap<String, CancellationToken>,
    operation_tokens: HashMap<String, CancellationToken>,
    pending_run_cancels: HashSet<String>,
    pending_operation_cancels: HashSet<String>,
}

impl TaskCancelRegistry {
    pub(super) fn register_run(&self, run_id: &str) -> CancellationToken {
        self.register(run_id, true)
    }

    pub(super) fn unregister_run(&self, run_id: &str) {
        self.unregister(run_id, true);
    }

    pub(super) fn cancel_run(&self, run_id: &str) -> bool {
        self.cancel(run_id, true)
    }

    pub(super) fn register_operation(&self, op_id: &str) -> CancellationToken {
        self.register(op_id, false)
    }

    pub(super) fn unregister_operation(&self, op_id: &str) {
        self.unregister(op_id, false);
    }

    pub(super) fn cancel_operation(&self, op_id: &str) -> bool {
        self.cancel(op_id, false)
    }

    fn register(&self, id: &str, is_run: bool) -> CancellationToken {
        let mut guard = self
            .inner
            .lock()
            .expect("task cancel registry mutex poisoned");
        let token = CancellationToken::new();

        if is_run {
            if let Some(prev) = guard.run_tokens.insert(id.to_string(), token.clone()) {
                prev.cancel();
            }
            if guard.pending_run_cancels.remove(id) {
                token.cancel();
            }
        } else {
            if let Some(prev) = guard.operation_tokens.insert(id.to_string(), token.clone()) {
                prev.cancel();
            }
            if guard.pending_operation_cancels.remove(id) {
                token.cancel();
            }
        }

        token
    }

    fn unregister(&self, id: &str, is_run: bool) {
        let mut guard = self
            .inner
            .lock()
            .expect("task cancel registry mutex poisoned");
        if is_run {
            let _ = guard.run_tokens.remove(id);
            let _ = guard.pending_run_cancels.remove(id);
        } else {
            let _ = guard.operation_tokens.remove(id);
            let _ = guard.pending_operation_cancels.remove(id);
        }
    }

    fn cancel(&self, id: &str, is_run: bool) -> bool {
        let mut guard = self
            .inner
            .lock()
            .expect("task cancel registry mutex poisoned");
        if is_run {
            if let Some(token) = guard.run_tokens.get(id) {
                token.cancel();
                return true;
            }
            guard.pending_run_cancels.insert(id.to_string());
            false
        } else {
            if let Some(token) = guard.operation_tokens.get(id) {
                token.cancel();
                return true;
            }
            guard.pending_operation_cancels.insert(id.to_string());
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaskCancelRegistry;

    #[test]
    fn cancel_run_before_register_is_sticky() {
        let registry = TaskCancelRegistry::default();
        assert!(!registry.cancel_run("run-1"));
        let token = registry.register_run("run-1");
        assert!(token.is_cancelled());
        registry.unregister_run("run-1");
    }

    #[test]
    fn cancel_operation_after_register_cancels_token() {
        let registry = TaskCancelRegistry::default();
        let token = registry.register_operation("op-1");
        assert!(!token.is_cancelled());
        assert!(registry.cancel_operation("op-1"));
        assert!(token.is_cancelled());
        registry.unregister_operation("op-1");
    }
}
