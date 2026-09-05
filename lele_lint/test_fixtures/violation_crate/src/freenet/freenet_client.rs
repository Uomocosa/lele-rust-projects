pub struct FreenetClient {
    pub token: String,
    pub peers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::FreenetClient;

    #[test]
    fn test_usage() {
        let client = FreenetClient {
            token: String::new(),
            peers: Vec::new(),
        };
        assert!(client.peers.is_empty());
    }
}
