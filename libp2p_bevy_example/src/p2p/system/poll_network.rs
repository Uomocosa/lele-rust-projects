use bevy::prelude::*;
use tracing::{debug, info, warn};

use crate::p2p;
use crate::p2p::resource::PeerState;
use crate::p2p::resource::Session;
use crate::p2p::Config;
use crate::p2p::NetworkEvent;
use crate::sync::resource::NetworkState;
use crate::sync::resource::RemoteInputBuffer;

pub(crate) fn can_accept_player(current_count: usize, config: &Config) -> bool {
    if let Some(max) = config.max_players {
        return current_count < max;
    }
    true
}

pub fn poll_network(
    mut swarm_state: ResMut<Session>,
    mut remote_buffer: ResMut<RemoteInputBuffer>,
    mut network_state: ResMut<NetworkState>,
    mut p2p_state: ResMut<PeerState>,
    mut events: MessageWriter<NetworkEvent>,
) {
    let auto_accept = swarm_state.swarm.config.auto_accept_join;
    let max_players = swarm_state.swarm.config.max_players;
    let can_accept = can_accept_player(p2p_state.connected_peers.len(), &swarm_state.swarm.config);

    while let Ok(event) = swarm_state.event_receiver.try_recv() {
        match event {
            NetworkEvent::PeerDiscovered(peer_id, addr) => {
                info!("Peer discovered: {} at {}", peer_id, addr);
                if !p2p_state.discovered_peers.contains(&peer_id) {
                    p2p_state.add_discovered_peer(peer_id);
                }
                if !network_state.discovered_peers.contains(&peer_id) {
                    network_state.discovered_peers.push(peer_id);
                }
                events.write(NetworkEvent::PeerDiscovered(peer_id, addr));
            }
            NetworkEvent::PeerConnected(peer_id) => {
                debug!("Peer connected: {}", peer_id);
                if !p2p_state.connected_peers.contains(&peer_id) {
                    if auto_accept && can_accept {
                        p2p_state.add_connected_peer(peer_id);
                    } else if !auto_accept {
                        p2p_state.add_join_request(peer_id);
                    } else {
                        warn!(
                            "Max players ({:?}) reached, rejecting connection from {}",
                            max_players, peer_id
                        );
                    }
                }
                if !network_state.connected_peers.contains(&peer_id) {
                    network_state.connected_peers.push(peer_id);
                }
                events.write(NetworkEvent::PeerConnected(peer_id));
            }
            NetworkEvent::PeerDisconnected(peer_id) => {
                debug!("Peer disconnected: {}", peer_id);
                if p2p_state.connected_peers.contains(&peer_id) {
                    p2p_state.remove_connected_peer(peer_id);
                }
                if p2p_state.pending_join_requests.contains(&peer_id) {
                    p2p_state.remove_join_request(peer_id);
                }
                network_state.connected_peers.retain(|p| *p != peer_id);
                events.write(NetworkEvent::PeerDisconnected(peer_id));
            }
            NetworkEvent::Message(peer_id, topic, data) => {
                if let Some(msg) = crate::sync::parse_message(&data) {
                    p2p::handle_incoming_message(&mut remote_buffer, peer_id, msg);
                }
                events.write(NetworkEvent::Message(peer_id, topic, data));
            }
            NetworkEvent::NewListenAddr(addr) => {
                info!("Listening on {}", addr);
                events.write(NetworkEvent::NewListenAddr(addr));
            }
        }
    }

    let connected_peers = swarm_state.swarm.get_connected_peers();

    for peer in connected_peers {
        if !p2p_state.connected_peers.contains(&peer) {
            if auto_accept && can_accept {
                p2p_state.add_connected_peer(peer);
            } else if !auto_accept {
                p2p_state.add_join_request(peer);
            } else {
                warn!(
                    "Max players ({:?}) reached, not accepting peer {}",
                    max_players, peer
                );
            }
            events.write(NetworkEvent::PeerConnected(peer));
        }
        if !network_state.connected_peers.contains(&peer) {
            network_state.connected_peers.push(peer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::can_accept_player;
    use crate::p2p::resource::PeerState;
    use crate::p2p::resource::Session;
    use crate::p2p::Config;
    use crate::p2p::Swarm;
    use crate::sync::resource::NetworkState;
    use crate::sync::resource::RemoteInputBuffer;
    use bevy::prelude::*;

    #[test]
    fn test_can_accept_player() {
        let unlimited = Config::default();
        assert!(can_accept_player(5, &unlimited));

        let limit_2 = Config::default().with_max_players(2);
        assert!(can_accept_player(1, &limit_2));
        assert!(!can_accept_player(2, &limit_2));
        assert!(!can_accept_player(3, &limit_2));
    }

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new();
        let config = Config::default();
        let (swarm, event_receiver) = Swarm::new(config.clone())?;
        let local_peer_id = swarm.local_peer_id;

        app.init_resource::<NetworkState>()
            .init_resource::<RemoteInputBuffer>()
            .insert_resource(Session {
                swarm,
                event_receiver,
            })
            .insert_resource(PeerState::new(&config, local_peer_id))
            .add_systems(FixedUpdate, crate::p2p::system::poll_network);

        app.update();
        app.update();

        let peer_state = app.world().resource::<PeerState>();
        assert!(peer_state.connected_peers.is_empty());
        Ok(())
    }
}
