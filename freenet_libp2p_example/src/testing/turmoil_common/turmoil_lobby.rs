use crate::testing;

#[must_use]
pub fn turmoil_lobby() -> String {
    testing::new_contract_params()
}

#[cfg(test)]
mod tests {
    use super::turmoil_lobby;

    #[test]
    fn test_usage() {
        assert_ne!(turmoil_lobby(), "");
    }
}
