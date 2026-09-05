pub fn load() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn test_usage() {
        assert!(load());
    }
}
