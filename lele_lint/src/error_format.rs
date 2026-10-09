use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ErrorFormat {
    Clippy,
    Github,
}

#[cfg(test)]
mod tests {
    use clap::ValueEnum;

    use super::ErrorFormat;

    #[test]
    fn test_usage() {
        assert_eq!(
            ErrorFormat::from_str("github", true),
            Ok(ErrorFormat::Github)
        );
        assert!(ErrorFormat::from_str("githb", true).is_err());
    }
}
