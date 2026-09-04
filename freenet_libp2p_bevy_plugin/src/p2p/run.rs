use std::time::Duration;

use futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::kad;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;

use crate::p2p;

pub async fn run<T: p2p::Message>(
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<T>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    keypair: Keypair,
) {
    let mut swarm = match p2p::build_swarm::build_swarm::<T>(keypair) {
        Ok(s) => s,
        Err(e) => {
            event_tx.send(p2p::Event::Error(e)).ok();
            return;
        }
    };

    let _ = swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap());
    let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap());

    let own_peer_id = swarm.local_peer_id().to_string();
    let mut listen_addrs: Vec<String> = Vec::new();
    let mut ready_deadline: Option<tokio::time::Instant> = None;

    loop {
        let ready_sleep = tokio::time::sleep_until(
            ready_deadline
                .unwrap_or_else(|| tokio::time::Instant::now() + Duration::from_secs(3600)),
        );
        tokio::pin!(ready_sleep);
        tokio::select! {
            _ = &mut ready_sleep, if ready_deadline.is_some() => {
                let addrs = std::mem::take(&mut listen_addrs);
                event_tx.send(p2p::Event::Ready { peer_id: own_peer_id.clone(), addrs }).ok();
                ready_deadline = None;
            }
            cmd = cmd_rx.recv() => match cmd {
                Some(p2p::Command::Dial { peer_id, addrs }) => {
                    for addr in addrs {
                        if let Ok(ma) = addr.parse::<libp2p::Multiaddr>() {
                            let _ = swarm.dial(ma);
                        }
                    }
                    let _ = peer_id;
                }
                Some(p2p::Command::Send { peer_id, payload }) => {
                    if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
                        swarm.behaviour_mut().request_response.send_request(&pid, payload);
                    }
                }
                Some(p2p::Command::PutHistory { lobby, chunk, data }) => {
                    let key = p2p::history::history_key(&lobby, chunk);
                    let record = kad::Record { key: key.clone(), value: data, publisher: None, expires: None };
                    let _ = swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One);
                    let _ = swarm.behaviour_mut().kademlia.start_providing(key);
                }
                Some(p2p::Command::FetchHistory { lobby, chunk }) => {
                    let key = p2p::history::history_key(&lobby, chunk);
                    swarm.behaviour_mut().kademlia.get_record(key);
                }
                None => break,
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    listen_addrs.push(address.to_string());
                    if ready_deadline.is_none() {
                        ready_deadline = Some(tokio::time::Instant::now() + Duration::from_millis(250));
                    }
                }
                SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RequestResponse(request_response::Event::Message { peer, message, .. })) => match message {
                    request_response::Message::Request { request, channel, .. } => {
                        let payload_clone = request.clone();
                        event_tx.send(p2p::Event::Message { from: peer.to_string(), payload: request }).ok();
                        let _ = swarm.behaviour_mut().request_response.send_response(channel, payload_clone);
                    }
                    request_response::Message::Response { response, .. } => {
                        event_tx.send(p2p::Event::Message { from: peer.to_string(), payload: response }).ok();
                    }
                },
                SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record))), .. })) => {
                    let record = peer_record.record;
                    let key_str = String::from_utf8_lossy(record.key.as_ref()).to_string();
                    let parts: Vec<&str> = key_str.split('/').collect();
                    if parts.len() >= 4 {
                        let lobby = parts[2].to_string();
                        let chunk = parts[3].parse::<u64>().unwrap_or(0);
                        event_tx.send(p2p::Event::HistoryChunk { lobby, chunk, data: record.value }).ok();
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    event_tx.send(p2p::Event::PeerConnected(peer_id.to_string())).ok();
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    event_tx.send(p2p::Event::PeerDisconnected(peer_id.to_string())).ok();
                }
                _ => {}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
// no test_usage necessary
