pub fn parse_freenet_local() -> bool {
    std::env::args().skip(1).any(|arg| arg == "--freenet-local")
}

#[cfg(test)]
mod tests {
    use super::parse_freenet_local;

    #[test]
    fn test_usage() {
        assert!(!parse_freenet_local());
    }
}
