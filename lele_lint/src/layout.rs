#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    Standard,
    Methods,
}

impl Layout {
    #[must_use]
    pub fn parse(value: &str) -> Layout {
        match value {
            "methods" => Layout::Methods,
            _ => Layout::Standard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Layout;

    #[test]
    fn test_usage() {
        assert_eq!(Layout::parse("methods"), Layout::Methods);
        assert_eq!(Layout::parse("standard"), Layout::Standard);
        assert_eq!(Layout::parse("nonsense"), Layout::Standard);
        assert_eq!(Layout::default(), Layout::Standard);
    }
}
