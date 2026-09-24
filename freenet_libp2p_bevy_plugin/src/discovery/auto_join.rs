#[must_use]
pub const fn auto_join(no_autojoin: Option<bool>) -> bool {
    no_autojoin.is_none()
}

#[cfg(test)]
mod tests {
    use super::auto_join;

    #[test]
    fn test_usage() {
        assert!(auto_join(None));
        assert!(!auto_join(Some(true)));
    }
}
