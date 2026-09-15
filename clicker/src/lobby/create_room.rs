#[must_use]
pub fn create_room(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return None;
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::create_room;

    #[test]
    fn test_usage() {
        assert_eq!(create_room("room-1"), Some("room-1".to_string()));
        assert_eq!(create_room("  room-1  "), Some("room-1".to_string()));
        assert_eq!(create_room(""), None);
        assert_eq!(create_room("   "), None);
        assert_eq!(create_room("has space"), None);
        assert_eq!(create_room("semi;colon"), None);
        assert_eq!(create_room(&"a".repeat(65)), None);
        assert_eq!(create_room(&"a".repeat(64)), Some("a".repeat(64)));
    }
}
