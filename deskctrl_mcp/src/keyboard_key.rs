use derive_more::Deref;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deref, Deserialize, schemars::JsonSchema)]
pub struct KeyboardKey(pub String);

#[cfg(test)]
mod tests {
    use crate::KeyboardKey;

    #[test]
    fn test_usage() {
        let key = KeyboardKey("ctrl".to_string());
        assert_eq!(key.as_str(), "ctrl");
    }
}
