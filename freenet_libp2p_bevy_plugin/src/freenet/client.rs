use derive_more::Deref;
use tokio::sync::mpsc;

#[derive(Debug, Deref)]
pub struct Client(pub mpsc::UnboundedSender<Vec<u8>>);

#[cfg(test)]
mod tests {
    use super::Client;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let c = Client(tx);
        assert!(!c.is_closed());
    }
}
