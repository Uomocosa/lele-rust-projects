use crate::clicker;

#[must_use]
pub fn contract_code_hash() -> [u8; 32] {
    *blake3::hash(clicker::contract_wasm()).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::contract_code_hash;

    #[test]
    fn test_usage() {
        let first = contract_code_hash();
        let again = contract_code_hash();
        assert_eq!(first, again);
        assert!(first.iter().any(|b| *b != 0));
    }
}
