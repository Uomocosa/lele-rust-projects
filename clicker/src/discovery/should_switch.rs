#[must_use]
pub fn should_switch(current: &str, requested: &str) -> bool {
    !requested.is_empty() && current != requested
}

#[cfg(test)]
mod tests {
    use super::should_switch;

    #[test]
    fn test_usage() {
        assert!(should_switch("room-a", "room-b"));
        assert!(!should_switch("room-a", "room-a"));
        assert!(!should_switch("room-a", ""));
    }
}
