use std::time::Duration;

use super::reconcile_env_from_env;

pub struct ReconcileEnv {
    pub machine: String,
    pub key: String,
    pub log_path: String,
    pub deadline: Duration,
    pub tag: u64,
}

impl Default for ReconcileEnv {
    fn default() -> Self {
        Self {
            machine: "linux".to_string(),
            key: "cross-os-default".to_string(),
            log_path: "cross-os-reconcile.log".to_string(),
            deadline: Duration::from_mins(15),
            tag: 1,
        }
    }
}

#[rustfmt::skip]
impl ReconcileEnv {
    #[must_use]
    pub fn from_env() -> Self { reconcile_env_from_env::from_env() }
}

// no test_usage necessary
