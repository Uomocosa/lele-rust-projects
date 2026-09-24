use crate::p2p;

use super::directory_state::DirectoryState;
use super::node_mode::NodeMode;
use super::player_id::PlayerId;

pub struct RunConfig {
    pub net_tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    pub tap_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    pub ready_rx: tokio::sync::watch::Receiver<Option<p2p::Ready>>,
    pub observed_rx: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    pub namespace: String,
    pub lobby: Option<String>,
    pub params_override: Option<String>,
    pub since_secs: u64,
    pub own: PlayerId,
    pub transport: p2p::TransportMode,
    pub node: NodeMode,
    pub room_tx: tokio::sync::watch::Sender<Option<String>>,
    pub room_requests: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub directory_tx: tokio::sync::mpsc::UnboundedSender<DirectoryState>,
    pub expected_tx: tokio::sync::mpsc::UnboundedSender<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::RunConfig;
    use crate::discovery;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (net_tx, _net_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_tap_tx, tap_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_ready_tx, ready_rx) = tokio::sync::watch::channel(None);
        let (_obs_tx, observed_rx) = tokio::sync::watch::channel(None);
        let (room_tx, _room_rx) = tokio::sync::watch::channel(None);
        let (_req_tx, room_requests) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (directory_tx, _directory_rx) =
            tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
        let (expected_tx, _expected_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        let config = RunConfig {
            net_tx,
            tap_rx,
            ready_rx,
            observed_rx,
            namespace: "blackboard-v1".to_string(),
            lobby: Some("room-20250101-120000".to_string()),
            params_override: None,
            since_secs: 0,
            own: discovery::PlayerId(1),
            transport: p2p::TransportMode::Both,
            node: discovery::NodeMode::Embedded,
            room_tx,
            room_requests,
            directory_tx,
            expected_tx,
        };
        assert_eq!(config.namespace, "blackboard-v1");
        assert_eq!(config.since_secs, 0);
    }
}
