use crate::p2p;

use super::super::directory::room_catalog::RoomCatalog;
use super::super::params::discovery_timing::DiscoveryTiming;
use super::super::params::freenet_endpoint::FreenetEndpoint;
use super::super::params::player_id::PlayerId;
use super::super::params::remote_peer_id::RemotePeerId;
use super::super::params::room_name::RoomName;
use super::super::params::unique_game_id::UniqueGameId;

pub struct RunConfig {
    pub net_tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    pub tap_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    pub ready_rx: tokio::sync::watch::Receiver<Option<p2p::Ready>>,
    pub observed_rx: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    pub id: UniqueGameId,
    pub timing: DiscoveryTiming,
    pub own: PlayerId,
    pub transport: p2p::TransportMode,
    pub room_tx: tokio::sync::watch::Sender<Option<RoomName>>,
    pub room_requests: tokio::sync::mpsc::UnboundedReceiver<RoomName>,
    pub directory_tx: tokio::sync::mpsc::UnboundedSender<RoomCatalog>,
    pub expected_tx: tokio::sync::mpsc::UnboundedSender<Vec<RemotePeerId>>,
    pub endpoint: FreenetEndpoint,
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
        let (_req_tx, room_requests) =
            tokio::sync::mpsc::unbounded_channel::<discovery::params::RoomName>();
        let (directory_tx, _directory_rx) =
            tokio::sync::mpsc::unbounded_channel::<discovery::directory::RoomCatalog>();
        let (expected_tx, _expected_rx) =
            tokio::sync::mpsc::unbounded_channel::<Vec<discovery::params::RemotePeerId>>();
        let config = RunConfig {
            net_tx,
            tap_rx,
            ready_rx,
            observed_rx,
            id: discovery::params::UniqueGameId::new(
                &discovery::params::GameName("test".to_string()),
                "token",
            ),
            timing: discovery::params::DiscoveryTiming::default(),
            own: discovery::params::PlayerId(1),
            transport: p2p::TransportMode::Both,
            room_tx,
            room_requests,
            directory_tx,
            expected_tx,
            endpoint: discovery::params::FreenetEndpoint(7509),
        };
        assert_eq!(config.id.as_str(), "test/token");
        assert_eq!(*config.endpoint, 7509);
    }
}
