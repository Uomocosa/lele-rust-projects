mod run_suite;
pub mod suite_config;

pub use run_suite::run_suite;
pub use suite_config::SuiteConfig;

use crate::suite_config as _suite_config_use;

fn _ensure_suite_config_used(_cfg: _suite_config_use::SuiteConfig) {}

// no test_usage necessary
