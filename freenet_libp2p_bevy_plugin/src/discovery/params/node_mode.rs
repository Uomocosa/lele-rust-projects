#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeMode {
    #[default]
    Embedded,
    External {
        ws_port: u16,
    },
}

#[cfg(test)]
mod tests {
    use super::NodeMode;

    #[test]
    fn test_usage() {
        assert_eq!(NodeMode::default(), NodeMode::Embedded);
        assert_ne!(NodeMode::Embedded, NodeMode::External { ws_port: 7509 });
    }
}
