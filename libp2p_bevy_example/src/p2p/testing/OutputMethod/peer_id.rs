use crate::p2p::testing::Output;

pub fn peer_id(output: &Output, tag: &str) -> Option<String> {
    let prefix = format!("{}:Evt:READY:", tag);
    output
        .lines
        .iter()
        .find_map(|line| line.strip_prefix(&prefix).map(|s| s.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::p2p::testing::Output;

    #[test]
    fn test_usage() {
        let output = Output {
            lines: vec![
                "A:Evt:READY:12D3KooWABC".to_string(),
                "B:Evt:READY:12D3KooWXYZ".to_string(),
            ],
        };
        assert_eq!(output.peer_id("A").as_deref(), Some("12D3KooWABC"));
        assert_eq!(output.peer_id("B").as_deref(), Some("12D3KooWXYZ"));
        assert_eq!(output.peer_id("C"), None);
    }
}
