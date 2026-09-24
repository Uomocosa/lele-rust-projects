#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ready {
    pub peer_id: String,
    pub addrs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::Ready;

    #[test]
    fn test_usage() {
        let ready = Ready {
            peer_id: "peer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
        };
        assert_eq!(ready.peer_id, "peer");
        assert_eq!(Ready::default().addrs.len(), 0);
    }
}
