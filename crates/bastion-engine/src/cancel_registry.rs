use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct CancelRegistry {
    run_tokens: Mutex<HashMap<String, CancellationToken>>,
    operation_tokens: Mutex<HashMap<String, CancellationToken>>,
}

impl CancelRegistry {
    pub fn register_run(&self, run_id: &str) -> CancellationToken {
        Self::register_token(&self.run_tokens, run_id)
    }

    pub fn unregister_run(&self, run_id: &str) {
        Self::unregister_token(&self.run_tokens, run_id);
    }

    pub fn cancel_run(&self, run_id: &str) -> bool {
        Self::cancel_token(&self.run_tokens, run_id)
    }

    pub fn register_operation(&self, op_id: &str) -> CancellationToken {
        Self::register_token(&self.operation_tokens, op_id)
    }

    pub fn unregister_operation(&self, op_id: &str) {
        Self::unregister_token(&self.operation_tokens, op_id);
    }

    pub fn cancel_operation(&self, op_id: &str) -> bool {
        Self::cancel_token(&self.operation_tokens, op_id)
    }

    fn register_token(
        map: &Mutex<HashMap<String, CancellationToken>>,
        id: &str,
    ) -> CancellationToken {
        let token = CancellationToken::new();
        let mut guard = lock_or_recover(map);
        if let Some(previous) = guard.insert(id.to_string(), token.clone()) {
            previous.cancel();
        }
        token
    }

    fn unregister_token(map: &Mutex<HashMap<String, CancellationToken>>, id: &str) {
        let mut guard = lock_or_recover(map);
        guard.remove(id);
    }

    fn cancel_token(map: &Mutex<HashMap<String, CancellationToken>>, id: &str) -> bool {
        let guard = lock_or_recover(map);
        let Some(token) = guard.get(id) else {
            return false;
        };
        token.cancel();
        true
    }
}

pub fn global_cancel_registry() -> &'static CancelRegistry {
    static REGISTRY: OnceLock<CancelRegistry> = OnceLock::new();
    REGISTRY.get_or_init(CancelRegistry::default)
}

fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::CancelRegistry;

    #[test]
    fn run_registration_cancel_and_unregistration_work() {
        let registry = CancelRegistry::default();
        let token = registry.register_run("run-1");
        assert!(!token.is_cancelled());

        assert!(registry.cancel_run("run-1"));
        assert!(token.is_cancelled());

        registry.unregister_run("run-1");
        assert!(!registry.cancel_run("run-1"));
    }

    #[test]
    fn operation_registration_cancel_and_unregistration_work() {
        let registry = CancelRegistry::default();
        let token = registry.register_operation("op-1");
        assert!(!token.is_cancelled());

        assert!(registry.cancel_operation("op-1"));
        assert!(token.is_cancelled());

        registry.unregister_operation("op-1");
        assert!(!registry.cancel_operation("op-1"));
    }
}
