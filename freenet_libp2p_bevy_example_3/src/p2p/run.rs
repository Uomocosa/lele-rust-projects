use std::time::Duration;

use futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId};

use crate::p2p;

pub async fn run(
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command>,
    event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event>,
    keypair: Keypair,
) {
    let mut swarm = match p2p::build_swarm::build_swarm(keypair) {
        Ok(swarm) => swarm,
        Err(e) => {
            event_tx
                .send(p2p::Event::Error(format!("swarm build failed: {e}")))
                .ok();
            return;
        }
    };

    if let Err(e) = listen(&mut swarm, "/ip4/0.0.0.0/udp/0/quic-v1") {
        event_tx.send(p2p::Event::Error(e.to_string())).ok();
        return;
    }
    if let Err(e) = listen(&mut swarm, "/ip4/0.0.0.0/tcp/0") {
        event_tx.send(p2p::Event::Error(e.to_string())).ok();
        return;
    }

    let own_peer_id = swarm.local_peer_id().to_base58();
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
                event_tx
                    .send(p2p::Event::Ready { peer_id: own_peer_id.clone(), addrs })
                    .ok();
                ready_deadline = None;
            }
            cmd = cmd_rx.recv() => match cmd {
                Some(p2p::Command::Dial { peer_id, addrs }) => {
                    if let Err(e) = peer_id.parse::<PeerId>() {
                        event_tx
                            .send(p2p::Event::Error(format!("invalid peer id {peer_id}: {e}")))
                            .ok();
                        continue;
                    }
                    tracing::info!(
                        target: "p2p::connect",
                        peer = %peer_id,
                        addrs = addrs.len(),
                        "dial attempt"
                    );
                    for addr in addrs {
                        match addr.parse::<Multiaddr>() {
                            Ok(multiaddr) => {
                                if let Err(e) = swarm.dial(multiaddr) {
                                    tracing::warn!(target: "p2p", peer_id = %peer_id, error = %e, "dial failed");
                                }
                            }
                            Err(e) => {
                                event_tx
                                    .send(p2p::Event::Error(format!("invalid multiaddr {addr}: {e}")))
                                    .ok();
                            }
                        }
                    }
                }
                Some(p2p::Command::SendNetcode { peer_id, msg }) => {
                    if let Ok(pid) = peer_id.parse::<PeerId>() {
                        swarm.behaviour_mut().netcode.send_request(&pid, msg);
                    }
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
                SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Netcode(
                    request_response::Event::Message { peer, message, .. },
                )) => match message {
                    request_response::Message::Request { request, channel, .. } => {
                        event_tx
                            .send(p2p::Event::IncomingNetcode { from: peer, msg: request })
                            .ok();
                        // Always respond (Experiment A). libp2p `request_response` keeps a
                        // request pending until it is answered; with thousands of unanswered
                        // requests per tick the connection degrades and inbound goes dead.
                        if swarm
                            .behaviour_mut()
                            .netcode
                            .send_response(channel, p2p::NetcodeMsg::Ack)
                            .is_ok()
                        {
                            tracing::trace!(target: "p2p::connect", peer = %peer, "netcode ack sent");
                        }
                    }
                    request_response::Message::Response { response, .. } => {
                        event_tx
                            .send(p2p::Event::IncomingNetcode { from: peer, msg: response })
                            .ok();
                    }
                },
                SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Netcode(
                    request_response::Event::OutboundFailure {
                        peer, request_id, error, ..
                    },
                )) => {
                    tracing::warn!(
                        target: "p2p::connect",
                        peer = %peer,
                        ?request_id,
                        error = %error,
                        "netcode outbound failure (request unanswered or peer unreachable)"
                    );
                }
                SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Netcode(
                    request_response::Event::InboundFailure {
                        peer, request_id, error, ..
                    },
                )) => {
                    tracing::warn!(
                        target: "p2p::connect",
                        peer = %peer,
                        ?request_id,
                        error = %error,
                        "netcode inbound failure"
                    );
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    tracing::info!(target: "p2p::connect", peer = %peer_id, "connection established");
                    event_tx.send(p2p::Event::PeerConnected(peer_id)).ok();
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    tracing::info!(target: "p2p::connect", peer = %peer_id, "connection closed");
                    event_tx.send(p2p::Event::PeerDisconnected(peer_id)).ok();
                }
                _ => {}
            },
        }
    }
}

// needed helper:
fn listen(swarm: &mut libp2p::Swarm<p2p::Behaviour>, addr: &str) -> Result<(), p2p::Error> {
    let multiaddr = addr
        .parse::<Multiaddr>()
        .map_err(|e| p2p::Error::Build(format!("invalid listen addr {addr}: {e}")))?;
    swarm
        .listen_on(multiaddr)
        .map(|_| ())
        .map_err(|e| p2p::Error::Swarm(format!("listen on {addr} failed: {e}")))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use libp2p::PeerId;
    use libp2p::identity::Keypair;

    use super::run;
    use crate::p2p;

    async fn wait_ready(
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event>,
    ) -> (String, Vec<String>) {
        loop {
            if let Some(p2p::Event::Ready { peer_id, addrs }) = rx.recv().await {
                return (peer_id, addrs);
            }
        }
    }

    async fn wait_connected(rx: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event>) -> PeerId {
        loop {
            if let Some(p2p::Event::PeerConnected(peer_id)) = rx.recv().await {
                return peer_id;
            }
        }
    }

    async fn wait_netcode(
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event>,
    ) -> p2p::NetcodeMsg {
        loop {
            if let Some(p2p::Event::IncomingNetcode { msg, .. }) = rx.recv().await
                && !matches!(msg, p2p::NetcodeMsg::Ack)
            {
                return msg;
            }
        }
    }

    #[tokio::test]
    async fn two_swarm_netcode_exchange() {
        let (cmd_tx_a, cmd_rx_a) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
        let (event_tx_a, mut event_rx_a) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        let (cmd_tx_b, cmd_rx_b) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
        let (event_tx_b, mut event_rx_b) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();

        let task_a = tokio::spawn(run(cmd_rx_a, event_tx_a, Keypair::generate_ed25519()));
        let task_b = tokio::spawn(run(cmd_rx_b, event_tx_b, Keypair::generate_ed25519()));

        let a_ready =
            tokio::time::timeout(Duration::from_secs(10), wait_ready(&mut event_rx_a)).await;
        assert!(a_ready.is_ok(), "swarm A never became ready");
        let (a_peer, a_addrs) = match a_ready {
            Ok((peer, addrs)) => (peer, addrs),
            Err(_) => return,
        };

        let b_ready =
            tokio::time::timeout(Duration::from_secs(10), wait_ready(&mut event_rx_b)).await;
        assert!(b_ready.is_ok(), "swarm B never became ready");
        let (b_peer, b_addrs) = match b_ready {
            Ok((peer, addrs)) => (peer, addrs),
            Err(_) => return,
        };

        let dial_addrs: Vec<String> = a_addrs
            .iter()
            .map(|addr| format!("{addr}/p2p/{a_peer}"))
            .collect();
        cmd_tx_b
            .send(p2p::Command::Dial {
                peer_id: a_peer.clone(),
                addrs: dial_addrs,
            })
            .ok();

        let a_saw_b =
            tokio::time::timeout(Duration::from_secs(10), wait_connected(&mut event_rx_a)).await;
        assert!(a_saw_b.is_ok(), "A never connected to B");

        let commit = p2p::NetcodeMsg::Commit {
            tick: 4,
            player_id: [2; 32],
            hash: 424242,
        };
        cmd_tx_b
            .send(p2p::Command::SendNetcode {
                peer_id: a_peer.clone(),
                msg: commit,
            })
            .ok();

        let a_recv =
            tokio::time::timeout(Duration::from_secs(10), wait_netcode(&mut event_rx_a)).await;
        assert!(a_recv.is_ok(), "A never received B's netcode commit");
        let a_recv = match a_recv {
            Ok(msg) => msg,
            Err(_) => return,
        };
        assert_eq!(a_recv, commit);

        let state_hash_msg = p2p::NetcodeMsg::StateHash { tick: 4, hash: 7 };
        cmd_tx_a
            .send(p2p::Command::SendNetcode {
                peer_id: b_peer.clone(),
                msg: state_hash_msg,
            })
            .ok();

        let b_recv =
            tokio::time::timeout(Duration::from_secs(10), wait_netcode(&mut event_rx_b)).await;
        assert!(b_recv.is_ok(), "B never received A's state hash");
        let b_recv = match b_recv {
            Ok(msg) => msg,
            Err(_) => return,
        };
        assert_eq!(b_recv, state_hash_msg);

        task_a.abort();
        task_b.abort();
        let _ = b_addrs;
        let _ = b_peer;
    }
}
// no test_usage necessary — real coverage is the two_swarm_netcode_exchange #[tokio::test]
