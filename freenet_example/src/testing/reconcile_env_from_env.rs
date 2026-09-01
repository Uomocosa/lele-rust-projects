use std::time::Duration;

use super::reconcile_env::ReconcileEnv;

#[must_use]
pub fn from_env() -> ReconcileEnv {
    let machine = std::env::var("CROSS_OS_MACHINE").unwrap_or_else(|_| "linux".into());
    let key = std::env::var("CROSS_OS_KEY").unwrap_or_else(|_| "cross-os-default".into());
    let log_path =
        std::env::var("CROSS_OS_LOG").unwrap_or_else(|_| "cross-os-reconcile.log".into());
    let deadline = Duration::from_secs(
        std::env::var("CROSS_OS_DEADLINE_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900),
    );
    let tag: u64 = match machine.as_str() {
        "windows" => 2,
        _ => 1,
    };
    ReconcileEnv {
        machine,
        key,
        log_path,
        deadline,
        tag,
    }
}

#[cfg(test)]
mod tests {
    use super::from_env;

    #[test]
    fn test_usage() {
        let env = from_env();
        assert!(!env.machine.is_empty());
        assert!(!env.key.is_empty());
    }
}
