use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::discovery;

pub struct RunConfig {
    pub cmd_tx: tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    pub ready: tokio::sync::watch::Receiver<Option<(String, Vec<String>)>>,
    pub observed: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    pub links: tokio::sync::mpsc::UnboundedReceiver<(String, bool)>,
    pub lobby_events: tokio::sync::mpsc::UnboundedReceiver<p2p::Event<clicker::CursorMsg>>,
    pub namespace: String,
    pub lobby: Option<String>,
    pub params_override: Option<String>,
    pub since_secs: u64,
    pub own: discovery::PlayerId,
    pub transport: p2p::TransportMode,
    pub room_tx: tokio::sync::watch::Sender<Option<String>>,
}
#[cfg(test)]
mod tests {
    use super::RunConfig;
    use crate::discovery;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_ready_tx, ready) = tokio::sync::watch::channel(None);
        let (_obs_tx, observed) = tokio::sync::watch::channel(None);
        let (_link_tx, links) = tokio::sync::mpsc::unbounded_channel();
        let (_lobby_tx, lobby_events) = tokio::sync::mpsc::unbounded_channel();
        let (room_tx, _room_rx) = tokio::sync::watch::channel(None);
        let config = RunConfig {
            cmd_tx,
            ready,
            observed,
            links,
            lobby_events,
            namespace: "blackboard-v1".to_string(),
            lobby: Some("room-20250101-120000".to_string()),
            params_override: None,
            since_secs: 0,
            own: discovery::PlayerId(1),
            transport: p2p::TransportMode::Both,
            room_tx,
        };
        assert_eq!(config.namespace, "blackboard-v1");
        assert_eq!(config.since_secs, 0);
    }
}
