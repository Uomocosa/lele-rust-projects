pub fn parse_freenet_gateway() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--freenet-gateway"
            && let Some(val) = args.next()
        {
            return Some(val);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_freenet_gateway;

    #[test]
    fn test_usage() {
        assert!(parse_freenet_gateway().is_none());
    }
}
