use derive_more::Deref;
use tokio::sync::mpsc;

#[derive(Debug, Deref)]
pub struct Client(pub mpsc::UnboundedSender<Vec<u8>>);
