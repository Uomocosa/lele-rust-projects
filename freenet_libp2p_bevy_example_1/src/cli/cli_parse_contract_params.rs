pub fn parse_contract_params() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--contract-params"
            && let Some(val) = args.next()
        {
            return Some(val);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_contract_params;

    #[test]
    fn test_usage() {
        assert!(parse_contract_params().is_none());
    }
}
