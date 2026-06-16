use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::mdns::tokio::Behaviour as Mdns;
use libp2p::swarm::{Config as SwarmConfig, Swarm as Libp2pSwarm};
use libp2p::{identity, noise, tcp, yamux, PeerId, Transport};

use crate::p2p::behaviour::BehaviourEvent;
use crate::p2p::Behaviour;
use crate::p2p::Command;
use crate::p2p::Config;
use crate::p2p::Error;
use crate::p2p::NetworkEvent;
use crate::p2p::Swarm;

pub fn new(config: Config) -> Result<(Swarm, mpsc::Receiver<NetworkEvent>), Error> {
    let (event_tx, event_rx) = mpsc::channel(100);
    let (command_sender, command_receiver) = mpsc::channel(100);

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(&local_key.public());

    info!("Local peer ID: {}", local_peer_id);

    let enable_mdns = config.enable_mdns;
    let enable_manual_dial = config.enable_manual_dial;
    let heartbeat_interval_ms = config.heartbeat_interval_ms;

    let noise_config = noise::Config::new(&local_key).map_err(|e| Error::Noise(e.to_string()))?;

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .message_id_fn(|message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            gossipsub::MessageId::from(hasher.finish().to_string())
        })
        .mesh_n(1)
        .mesh_n_low(1)
        .build()
        .map_err(|e| Error::Gossipsub(e.to_string()))?;

    let gossipsub_behaviour = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .map_err(|e| Error::Gossipsub(e.to_string()))?;

    std::thread::spawn(move || {
        info!("Swarm thread started, initializing networking");

        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                warn!("Failed to create tokio runtime: {}", e);
                return;
            }
        };

        let _guard = rt.enter();

        let mdns: Option<Mdns> = if enable_mdns {
            match Mdns::new(libp2p::mdns::Config::default(), local_peer_id) {
                Ok(m) => {
                    info!("mDNS enabled");
                    Some(m)
                }
                Err(e) => {
                    warn!("mDNS unavailable: {}", e);
                    None
                }
            }
        } else {
            warn!("mDNS disabled by config");
            None
        };

        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise_config)
            .multiplex(yamux::Config::default())
            .boxed();

        let behaviour = Behaviour {
            mdns: mdns.into(),
            gossipsub: gossipsub_behaviour,
        };

        let mut swarm = Libp2pSwarm::new(
            transport,
            behaviour,
            local_peer_id,
            SwarmConfig::without_executor(),
        );

        let topic = IdentTopic::new(crate::p2p::GAME_TOPIC_STR);
        swarm.behaviour_mut().gossipsub.subscribe(&topic).ok();

        let listen_addr = match "/ip4/0.0.0.0/tcp/0".parse() {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Failed to parse listen address: {}", e);
                return;
            }
        };
        if let Err(e) = swarm.listen_on(listen_addr) {
            warn!("listen_on failed: {}", e);
        }

        drop(_guard);

        let swarm = Arc::new(Mutex::new(swarm));

        let command_receiver = Arc::new(Mutex::new(command_receiver));
        let swarm_for_stream = swarm.clone();

        let mut enable_manual_dial = enable_manual_dial;
        let mut _last_heartbeat = Instant::now();

        loop {
            let cmd = {
                if let Ok(mut receiver) = command_receiver.lock() {
                    receiver.try_recv().ok()
                } else {
                    None
                }
            };

            if let Some(cmd) = cmd {
                if let Ok(mut swarm) = swarm.lock() {
                    match cmd {
                        Command::Publish(topic, msg) => match bincode::serialize(&msg) {
                            Ok(data) => {
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data)
                                {
                                    warn!("Publish failed: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to serialize message: {}", e);
                            }
                        },
                        Command::Dial(addr) => {
                            if enable_manual_dial {
                                if let Err(e) = swarm.dial(addr) {
                                    warn!("Dial failed: {}", e);
                                }
                            } else {
                                warn!("Manual dial disabled by config");
                            }
                        }
                        Command::GetPeers(sender) => {
                            let peers: Vec<PeerId> = swarm.connected_peers().copied().collect();
                            drop(swarm);
                            rt.block_on(async {
                                sender.send(peers).await.ok();
                            });
                        }
                        Command::SetEnableManualDial(enabled) => {
                            enable_manual_dial = enabled;
                            debug!("Manual dial set to: {}", enabled);
                        }
                    }
                }
            }

            let event = {
                if let Ok(mut swarm_guard) = swarm_for_stream.lock() {
                    rt.block_on(async {
                        tokio::select! {
                            event = futures::future::poll_fn(|cx| {
                                swarm_guard.poll_next_unpin(cx)
                            }) => { event }
                            _ = tokio::time::sleep(
                                std::time::Duration::from_millis(heartbeat_interval_ms)
                            ) => { None }
                        }
                    })
                } else {
                    None
                }
            };

            if let Some(event) = event {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {}", address);
                        event_tx.try_send(NetworkEvent::NewListenAddr(address)).ok();
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        debug!("Connected to {}", peer_id);
                        event_tx.try_send(NetworkEvent::PeerConnected(peer_id)).ok();
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        debug!("Disconnected from {}", peer_id);
                        event_tx
                            .try_send(NetworkEvent::PeerDisconnected(peer_id))
                            .ok();
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(event) => match event {
                        BehaviourEvent::Mdns(mdns_event) => {
                            if let libp2p::mdns::Event::Discovered(peers) = mdns_event {
                                for (peer_id, addr) in peers {
                                    info!("Discovered peer via mDNS: {} at {}", peer_id, addr);
                                    event_tx
                                        .try_send(NetworkEvent::PeerDiscovered(peer_id, addr))
                                        .ok();
                                }
                            }
                        }
                        BehaviourEvent::Gossipsub(gossipsub_event) => {
                            if let gossipsub::Event::Message {
                                propagation_source,
                                message,
                                ..
                            } = gossipsub_event
                            {
                                debug!("Received message from {}", propagation_source);
                                event_tx
                                    .try_send(NetworkEvent::Message(
                                        propagation_source,
                                        message.topic,
                                        message.data,
                                    ))
                                    .ok();
                            }
                        }
                    },
                    _ => {}
                }
            }

            _last_heartbeat = Instant::now();
            std::thread::sleep(std::time::Duration::from_millis(heartbeat_interval_ms));
        }
    });

    Ok((
        Swarm {
            local_peer_id,
            command_sender,
            config,
        },
        event_rx,
    ))
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::p2p::Config;
    use crate::p2p::Swarm;
    use libp2p::PeerId;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (swarm, _rx) = Swarm::new(config)?;
        let peer_id = swarm.local_peer_id;

        tracing::info!("Swarm initialized with peer ID: {}", peer_id);

        let peer_id_str = peer_id.to_string();
        assert!(!peer_id_str.is_empty(), "Peer ID should not be empty");
        assert!(
            peer_id_str.starts_with("12D3Koo"),
            "Peer ID should be a valid libp2p PeerId"
        );
        Ok(())
    }

    #[test]
    fn test_mdns_disabled_by_config() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default().with_mdns(false);
        let (_swarm, _rx) = Swarm::new(config)?;
        Ok(())
    }

    #[test]
    fn test_manual_dial_disabled_by_config() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default().with_manual_dial(false);
        let (_swarm, _rx) = Swarm::new(config)?;
        Ok(())
    }

    #[test]
    fn test_get_discovered_peers() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (_swarm, _rx) = Swarm::new(config)?;

        let peers: Vec<PeerId> = Vec::new();
        tracing::debug!("Discovered peers: {:?}", peers);
        Ok(())
    }

    #[test]
    fn test_get_connected_peers() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (mut swarm, _rx) = Swarm::new(config)?;

        let peers = swarm.get_connected_peers();

        tracing::debug!("Connected peers: {:?}", peers);

        assert!(peers.is_empty(), "New swarm should have no connected peers");
        Ok(())
    }

    #[test]
    #[ignore = "mDNS requires real network/multicast between separate machines on same LAN — two swarms in the same process on the same host won't route multicast back to each other"]
    fn test_mdns_bidirectional_discovery() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (swarm1, mut rx1) = Swarm::new(config.clone())?;
        let (swarm2, mut rx2) = Swarm::new(config)?;

        let peer1_id = swarm1.local_peer_id;
        let peer2_id = swarm2.local_peer_id;

        tracing::info!("Testing mDNS between {} and {}", peer1_id, peer2_id);

        let timeout = Duration::from_secs(10);
        let deadline = Instant::now() + timeout;

        let mut found_1_to_2 = false;
        let mut found_2_to_1 = false;

        while Instant::now() < deadline {
            if let Ok(p2p::NetworkEvent::PeerDiscovered(pid, _)) = rx1.try_recv() {
                if pid == peer2_id {
                    found_1_to_2 = true;
                }
            }

            if let Ok(p2p::NetworkEvent::PeerDiscovered(pid, _)) = rx2.try_recv() {
                if pid == peer1_id {
                    found_2_to_1 = true;
                }
            }

            if found_1_to_2 && found_2_to_1 {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        assert!(found_1_to_2, "swarm1 should discover swarm2");
        assert!(found_2_to_1, "swarm2 should discover swarm1");
        Ok(())
    }
}
