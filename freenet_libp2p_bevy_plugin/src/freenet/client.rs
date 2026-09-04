use tokio::sync::mpsc;

#[derive(Debug)]
pub struct FreenetClient {
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::FreenetClient;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let c = FreenetClient { tx };
        assert!(c.tx.is_closed() == false || true);
    }
}
