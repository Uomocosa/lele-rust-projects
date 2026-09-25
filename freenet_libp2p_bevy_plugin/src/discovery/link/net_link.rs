use crate::p2p;

pub struct NetLink {
    pub tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    pub observed: tokio::sync::watch::Receiver<Option<Vec<String>>>,
}
// no test_usage necessary
