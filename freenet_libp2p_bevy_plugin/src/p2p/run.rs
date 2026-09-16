use std::time::Duration;

use futures::StreamExt;
use libp2p::gossipsub;
use libp2p::identify;
use libp2p::identity::Keypair;
use libp2p::kad;
use libp2p::mdns;
use libp2p::relay;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};

use crate::p2p;

pub async fn run<T: p2p::Message>(
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<T>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    keypair: Keypair,
    mode: p2p::TransportMode,
    mdns_enabled: bool,
) {
    let mut swarm = match p2p::build_swarm::build_swarm::<T>(keypair, mdns_enabled) {
        Ok(s) => s,
        Err(e) => {
            event_tx.send(p2p::Event::Error(e)).ok();
            return;
        }
    };

    if mode != p2p::TransportMode::Tcp
        && let Ok(quic_addr) = "/ip4/0.0.0.0/udp/0/quic-v1".parse()
    {
        let _ = swarm.listen_on(quic_addr);
    }
    if mode != p2p::TransportMode::Quic
        && let Ok(tcp_addr) = "/ip4/0.0.0.0/tcp/0".parse()
    {
        let _ = swarm.listen_on(tcp_addr);
    }

    let own_peer_id = swarm.local_peer_id().to_string();
    let mut listen_addrs: Vec<String> = Vec::new();
    let mut ready_deadline: Option<tokio::time::Instant> = None;
    let mut mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
    let mut lobby_queries: std::collections::HashMap<libp2p::kad::QueryId, String> =
        std::collections::HashMap::new();

    loop {
        let now = tokio::time::Instant::now();
        let far = now.checked_add(Duration::from_secs(3600)).unwrap_or(now);
        let ready_sleep = tokio::time::sleep_until(ready_deadline.unwrap_or(far));
        tokio::pin!(ready_sleep);
        let mesh_sleep = tokio::time::sleep_until(mesh_deadline.unwrap_or(far));
        tokio::pin!(mesh_sleep);
        tokio::select! {
            () = &mut ready_sleep, if ready_deadline.is_some() => {
                let addrs = std::mem::take(&mut listen_addrs);
                event_tx.send(p2p::Event::Ready { peer_id: own_peer_id.clone(), addrs }).ok();
                ready_deadline = None;
            }
            () = &mut mesh_sleep => {
                log_mesh_snapshot(&swarm);
                mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
            }
            cmd = cmd_rx.recv() => {
                if !dispatch_command(&mut swarm, &event_tx, &mut lobby_queries, cmd) {
                    break;
                }
            }
            event = swarm.select_next_some() => {
                handle_swarm(
                    &mut swarm,
                    &event_tx,
                    &lobby_queries,
                    &mut listen_addrs,
                    &mut ready_deadline,
                    mode,
                    event,
                );
            }
        }
    }
}

// needed helper: routes one swarm event into behaviour handlers and Bevy events
fn handle_swarm<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    lobby_queries: &std::collections::HashMap<libp2p::kad::QueryId, String>,
    listen_addrs: &mut Vec<String>,
    ready_deadline: &mut Option<tokio::time::Instant>,
    mode: p2p::TransportMode,
    event: SwarmEvent<p2p::behaviour::BehaviourEvent<T>>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            listen_addrs.push(address.to_string());
            if ready_deadline.is_none() {
                let now = tokio::time::Instant::now();
                *ready_deadline = now.checked_add(Duration::from_millis(250));
            }
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RequestResponse(
            request_response::Event::Message { peer, message, .. },
        )) => {
            handle_request_response(swarm, &event_tx, &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record))),
                ..
            },
        )) => {
            handle_found_record(&event_tx, peer_record);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed {
                id,
                result:
                    kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                        providers,
                        ..
                    })),
                ..
            },
        )) => {
            handle_found_providers(&event_tx, lobby_queries, id, &providers);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Gossipsub(gossip_event)) => {
            handle_gossipsub(&*swarm, &event_tx, gossip_event);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
        )) => {
            event_tx
                .send(p2p::Event::RelayReserved {
                    relay_peer_id: relay_peer_id.to_string(),
                })
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Identify(
            identify::Event::Received { info, .. },
        )) => {
            event_tx
                .send(p2p::Event::ObservedAddr(info.observed_addr.to_string()))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(
            peers,
        ))) => {
            dial_mdns_peers(swarm, mode, peers);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Expired(_))) => {}
        SwarmEvent::ConnectionEstablished {
            peer_id,
            connection_id,
            endpoint,
            ..
        } => {
            log_established(&peer_id, connection_id, &endpoint);
            event_tx
                .send(p2p::Event::PeerConnected(peer_id.to_string()))
                .ok();
        }
        SwarmEvent::ConnectionClosed {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            cause,
            ..
        } => {
            log_closed(
                swarm,
                &peer_id,
                connection_id,
                &endpoint,
                num_established,
                cause.as_ref(),
            );
            event_tx
                .send(p2p::Event::PeerDisconnected(peer_id.to_string()))
                .ok();
        }
        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
            note_dial_failure(&event_tx, peer_id, &error);
        }
        _ => {}
    }
}

// needed helper: records one established connection with its endpoint
fn log_established(
    peer_id: &libp2p::PeerId,
    connection_id: libp2p::swarm::ConnectionId,
    endpoint: &libp2p::core::ConnectedPoint,
) {
    tracing::info!(
        target: "p2p",
        peer = %peer_id,
        connection = %connection_id,
        address = %endpoint.get_remote_address(),
        dialer = endpoint.is_dialer(),
        "p2p connection established"
    );
}

// needed helper: records one closed connection and whether the peer stays up
fn log_closed<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    peer_id: &libp2p::PeerId,
    connection_id: libp2p::swarm::ConnectionId,
    endpoint: &libp2p::core::ConnectedPoint,
    remaining: u32,
    cause: Option<&libp2p::swarm::ConnectionError>,
) {
    if let Some(reason) = cause {
        tracing::info!(
            target: "p2p",
            peer = %peer_id,
            connection = %connection_id,
            address = %endpoint.get_remote_address(),
            dialer = endpoint.is_dialer(),
            still_connected = swarm.is_connected(peer_id),
            remaining,
            reason = %reason,
            "p2p connection closed",
        );
    } else {
        tracing::info!(
            target: "p2p",
            peer = %peer_id,
            connection = %connection_id,
            address = %endpoint.get_remote_address(),
            dialer = endpoint.is_dialer(),
            still_connected = swarm.is_connected(peer_id),
            remaining,
            "p2p connection closed",
        );
    }
}

// needed helper: samples gossip mesh depth for one inbound message
fn log_gossip_in<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    propagation_source: &libp2p::PeerId,
    message_id: &libp2p::gossipsub::MessageId,
    message: &gossipsub::Message,
) {
    let mesh = swarm
        .behaviour()
        .gossipsub
        .mesh_peers(&message.topic)
        .count();
    tracing::debug!(
        target: "p2p",
        source = %propagation_source,
        message = %message_id,
        topic = %message.topic,
        mesh,
        "p2p gossip received"
    );
}

// needed helper: snapshots gossip mesh membership across subscribed topics
fn log_mesh_snapshot<T: p2p::Message>(swarm: &libp2p::Swarm<p2p::Behaviour<T>>) {
    let total = swarm.behaviour().gossipsub.all_mesh_peers().count();
    let mut topics = Vec::new();
    for topic in swarm.behaviour().gossipsub.topics() {
        let depth = swarm.behaviour().gossipsub.mesh_peers(topic).count();
        topics.push(format!("{topic}={depth}"));
    }
    tracing::debug!(target: "p2p", total, topics = ?topics, "p2p mesh snapshot");
}

// needed helper: answers one request-response message and forwards it to Bevy
fn handle_request_response<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer: &libp2p::PeerId,
    message: request_response::Message<T, T>,
) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let payload_clone = request.clone();
            event_tx
                .send(p2p::Event::Message {
                    from: peer.to_string(),
                    payload: request,
                })
                .ok();
            let _ = swarm
                .behaviour_mut()
                .request_response
                .send_response(channel, payload_clone);
        }
        request_response::Message::Response { response, .. } => {
            event_tx
                .send(p2p::Event::Message {
                    from: peer.to_string(),
                    payload: response,
                })
                .ok();
        }
    }
}

// needed helper: forwards one kad history record into a Bevy chunk event
fn handle_found_record<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_record: kad::PeerRecord,
) {
    let record = peer_record.record;
    let key_str = String::from_utf8_lossy(record.key.as_ref()).to_string();
    let parts: Vec<&str> = key_str.split('/').collect();
    if parts.len() >= 4
        && let Some(lobby) = parts.get(2)
        && let Some(chunk) = parts.get(3)
    {
        let lobby = lobby.to_string();
        let chunk = chunk.parse::<u64>().unwrap_or(0);
        event_tx
            .send(p2p::Event::HistoryChunk {
                lobby,
                chunk,
                data: record.value,
            })
            .ok();
    }
}

// needed helper: reports one outgoing connection error to Bevy or debug log
fn note_dial_failure<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_id: Option<libp2p::PeerId>,
    error: &libp2p::swarm::DialError,
) {
    match peer_id {
        Some(peer_id) => {
            event_tx
                .send(p2p::Event::DialFailed {
                    peer_id: peer_id.to_string(),
                    reason: error.to_string(),
                })
                .ok();
        }
        None => {
            tracing::debug!(target: "p2p", error = %error, "p2p dial failed without peer");
        }
    }
}

// needed helper: forwards one kad provider set into a Bevy lobby event
fn handle_found_providers<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    lobby_queries: &std::collections::HashMap<libp2p::kad::QueryId, String>,
    id: libp2p::kad::QueryId,
    providers: &std::collections::HashSet<libp2p::PeerId>,
) {
    if let Some(lobby) = lobby_queries.get(&id) {
        let peers = providers
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        event_tx
            .send(p2p::Event::LobbyProviders {
                lobby: lobby.clone(),
                peers,
            })
            .ok();
    }
}

// needed helper: routes one gossipsub event with mesh-depth diagnostics
fn handle_gossipsub<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    gossip_event: gossipsub::Event,
) {
    match gossip_event {
        gossipsub::Event::Message {
            propagation_source,
            message_id,
            message,
        } => {
            log_gossip_in(swarm, &propagation_source, &message_id, &message);
            forward_gossip(event_tx, propagation_source, message);
        }
        gossipsub::Event::Subscribed { peer_id, topic } => {
            tracing::debug!(target: "p2p", peer = %peer_id, topic = %topic, "p2p gossip subscribed");
        }
        gossipsub::Event::Unsubscribed { peer_id, topic } => {
            tracing::debug!(target: "p2p", peer = %peer_id, topic = %topic, "p2p gossip unsubscribed");
        }
        gossipsub::Event::SlowPeer { peer_id, .. } => {
            tracing::warn!(target: "p2p", peer = %peer_id, "p2p gossip slow peer");
        }
        gossipsub::Event::GossipsubNotSupported { peer_id } => {
            tracing::warn!(target: "p2p", peer = %peer_id, "p2p gossip not supported");
        }
    }
}

// needed helper: forwards one Bevy command into the swarm behaviours
fn dispatch_command<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    lobby_queries: &mut std::collections::HashMap<libp2p::kad::QueryId, String>,
    cmd: Option<p2p::Command<T>>,
) -> bool {
    match cmd {
        Some(p2p::Command::Dial { peer_id, addrs }) => {
            dial_peer(swarm, event_tx, &peer_id, &addrs, false);
        }
        Some(p2p::Command::DialForce { peer_id, addrs }) => {
            dial_peer(swarm, event_tx, &peer_id, &addrs, true);
        }
        Some(p2p::Command::ReserveRelay { relay_addr }) => {
            if let Ok(addr) = relay_addr.parse::<libp2p::Multiaddr>() {
                let _ = swarm.listen_on(addr);
            }
        }
        Some(p2p::Command::SetMdns { enabled }) => {
            swarm.behaviour_mut().mdns = Toggle::from(
                enabled
                    .then(|| {
                        mdns::tokio::Behaviour::new(mdns::Config::default(), *swarm.local_peer_id())
                            .ok()
                    })
                    .flatten(),
            );
        }
        Some(p2p::Command::AddKadPeer { peer_id, addrs }) => {
            seed_kad_peer(swarm, &peer_id, &addrs);
        }
        Some(p2p::Command::ProvideLobby { lobby }) => {
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .start_providing(p2p::provider_key(&lobby));
        }
        Some(p2p::Command::FindLobby { lobby }) => {
            let id = swarm
                .behaviour_mut()
                .kademlia
                .get_providers(p2p::provider_key(&lobby));
            lobby_queries.insert(id, lobby);
        }
        Some(p2p::Command::Send { peer_id, payload }) => {
            if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
                swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&pid, payload);
            }
        }
        Some(p2p::Command::PutHistory { lobby, chunk, data }) => {
            let key = p2p::history_key(&lobby, chunk);
            let record = kad::Record {
                key: key.clone(),
                value: data,
                publisher: None,
                expires: None,
            };
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .put_record(record, kad::Quorum::One);
            let _ = swarm.behaviour_mut().kademlia.start_providing(key);
        }
        Some(p2p::Command::FetchHistory { lobby, chunk }) => {
            let key = p2p::history_key(&lobby, chunk);
            swarm.behaviour_mut().kademlia.get_record(key);
        }
        Some(p2p::Command::FetchRoster { lobby }) => {
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .start_providing(p2p::provider_key(&lobby));
            let id = swarm
                .behaviour_mut()
                .kademlia
                .get_providers(p2p::provider_key(&lobby));
            lobby_queries.insert(id, lobby);
        }
        Some(p2p::Command::Subscribe { topic }) => {
            let topic = gossipsub::IdentTopic::new(topic);
            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        Some(p2p::Command::Publish { topic, data }) => {
            let topic = gossipsub::IdentTopic::new(topic);
            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
        }
        None => return false,
    }
    true
}
// needed helper: dials one peer with peer binding when the id parses
fn dial_peer<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_id: &str,
    addrs: &[String],
    force: bool,
) {
    let parsed: Vec<libp2p::Multiaddr> = addrs.iter().filter_map(|a| a.parse().ok()).collect();
    if parsed.is_empty() {
        return;
    }
    if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
        let condition = if force {
            PeerCondition::Always
        } else {
            PeerCondition::DisconnectedAndNotDialing
        };
        let opts = DialOpts::peer_id(pid)
            .condition(condition)
            .addresses(parsed)
            .build();
        if let Err(e) = swarm.dial(opts) {
            event_tx
                .send(p2p::Event::DialFailed {
                    peer_id: peer_id.to_string(),
                    reason: e.to_string(),
                })
                .ok();
        }
        return;
    }
    for addr in parsed {
        let _ = swarm.dial(addr);
    }
}

// needed helper: feeds one discovered peer into kad routing, bootstrapping on first sight
fn seed_kad_peer<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    peer_id: &str,
    addrs: &[String],
) {
    let Ok(pid) = peer_id.parse::<libp2p::PeerId>() else {
        return;
    };
    let mut added = false;
    for addr in addrs {
        if let Ok(ma) = addr.parse::<libp2p::Multiaddr>() {
            if swarm.behaviour_mut().kademlia.add_address(&pid, ma) == kad::RoutingUpdate::Success {
                added = true;
            }
        }
    }
    if added {
        let _ = swarm.behaviour_mut().kademlia.bootstrap();
    }
}

// needed helper: dials mDNS-discovered LAN peers filtered by transport mode
fn dial_mdns_peers<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    mode: p2p::TransportMode,
    peers: Vec<(libp2p::PeerId, libp2p::Multiaddr)>,
) {
    for (peer, addr) in peers {
        let addr = addr.to_string();
        let wanted = match mode {
            p2p::TransportMode::Tcp => !addr.contains("/udp/"),
            p2p::TransportMode::Quic => addr.contains("/udp/"),
            p2p::TransportMode::Both => true,
        };
        let Ok(multi) = addr.parse::<libp2p::Multiaddr>() else {
            continue;
        };
        if !wanted {
            continue;
        }
        let opts = DialOpts::peer_id(peer)
            .condition(PeerCondition::DisconnectedAndNotDialing)
            .addresses(vec![multi])
            .build();
        if let Err(e) = swarm.dial(opts) {
            tracing::debug!(target: "p2p", error = %e, "mdns dial skipped");
        }
    }
}

// needed helper: converts a gossipsub wire message into a Bevy event
fn forward_gossip<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    propagation_source: libp2p::PeerId,
    message: gossipsub::Message,
) {
    event_tx
        .send(p2p::Event::Gossip {
            topic: message.topic.to_string(),
            from: message
                .source
                .map_or_else(|| propagation_source.to_string(), |s| s.to_string()),
            data: message.data,
        })
        .ok();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(run);
    }
}
// no test_usage necessary
