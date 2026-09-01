use std::time::{SystemTime, UNIX_EPOCH};

pub fn new_contract_params() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("local-mainnet-{secs}-{}", rand_suffix())
}

fn rand_suffix() -> u32 {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    n % 100_000
}

#[cfg(test)]
mod tests {
    use super::new_contract_params;

    #[test]
    fn test_usage() {
        let a = new_contract_params();
        let b = new_contract_params();
        assert!(a.starts_with("local-mainnet-"));
        assert_ne!(a, b);
    }
}
