use derive_more::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref)]
pub struct GameName(pub String);

#[cfg(test)]
mod tests {
    use super::GameName;

    #[test]
    fn test_usage() {
        let name = GameName("chess".to_string());
        assert_eq!(name.as_str(), "chess");
        assert_ne!(name, GameName("checkers".to_string()));
    }
}
