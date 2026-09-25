use derive_more::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref)]
pub struct GameToken(pub String);

#[cfg(test)]
mod tests {
    use super::GameToken;

    #[test]
    fn test_usage() {
        let token = GameToken("my-crate".to_string());
        assert_eq!(token.as_str(), "my-crate");
    }
}
