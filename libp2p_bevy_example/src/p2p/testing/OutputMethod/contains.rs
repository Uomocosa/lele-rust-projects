use crate::p2p::testing::Output;

pub fn contains(output: &Output, line: &str) -> bool {
    output.lines.iter().any(|l| l == line)
}

#[cfg(test)]
mod tests {
    use crate::p2p::testing::Output;

    #[test]
    fn test_usage() {
        let output = Output {
            lines: vec!["hello".to_string(), "world".to_string()],
        };
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        assert!(!output.contains("nope"));
    }
}
