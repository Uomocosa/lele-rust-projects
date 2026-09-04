#[must_use]
pub fn new_contract_params() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("lobby-{:x}", nanos & 0xffff_ffff)
}

#[cfg(test)]
mod tests {
    use super::new_contract_params;

    #[test]
    fn test_usage() {
        let a = new_contract_params();
        let b = new_contract_params();
        assert_ne!(a, "");
        assert_ne!(a, b);
    }
}
