use std::path::PathBuf;

pub fn parse_identity_dir() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--identity-dir"
            && let Some(val) = args.next()
        {
            return Some(PathBuf::from(val));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_identity_dir;

    #[test]
    fn test_usage() {
        assert!(parse_identity_dir().is_none());
    }
}
