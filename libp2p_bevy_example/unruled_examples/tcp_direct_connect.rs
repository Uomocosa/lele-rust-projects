// This file is in `unruled_examples/` — it is EXEMPT from the project's
// syntax and architecture rules in `.agents/rules/`. Only files inside
// `unruled_examples/` share this exemption. Refactor to follow the rules
// before promoting to `examples/` or `src/`.
//
// OBJECTIVE: Validate that two swarms in the same process can connect via
// direct TCP dial (no mDNS) and exchange a gossipsub Message::Ping over
// the game topic. Serves as the baseline test for transport connectivity
// — if this passes but mDNS-based tests fail, the bug is in the mDNS dial
// path (e.g. how /p2p/<peer_id> addresses are handled during dial).

use std::time::{Duration, Instant};

use bevy_p2p_app::p2p::{self, Config, Message, NetworkEvent, Swarm};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // --- Swarm A: listen and report address ---
    let config_a = Config::new().with_mdns(false).with_heartbeat(100);
    let (swarm_a, mut rx_a) = Swarm::new(config_a)?;
    let peer_a = swarm_a.local_peer_id;
    tracing::info!("Swarm A: {}", peer_a);

    let listen_addr = wait_for_event(&mut rx_a, |e| {
        if let NetworkEvent::NewListenAddr(addr) = e {
            Some(addr.clone())
        } else {
            None
        }
    })
    .expect("Swarm A should get a listen address within 10 s");
    tracing::info!("Swarm A listening on: {}", listen_addr);

    // --- Swarm B: dial A and verify connection ---
    let config_b = Config::new().with_mdns(false).with_heartbeat(100);
    let (mut swarm_b, mut rx_b) = Swarm::new(config_b)?;
    let peer_b = swarm_b.local_peer_id;
    tracing::info!("Swarm B: {}", peer_b);

    // Discover A's actual TCP port from its NewListenAddr.
    // Swarm A may have multiple addresses; pick the first TCP one.
    swarm_b.dial(listen_addr);

    // Wait for B→A connection
    let connected = wait_for_event(&mut rx_b, |e| {
        if let NetworkEvent::PeerConnected(pid) = e {
            Some(*pid)
        } else {
            None
        }
    })
    .expect("Swarm B should connect to A within 10 s");
    assert_eq!(connected, peer_a, "B should be connected to A");
    tracing::info!("Swarm B connected to A");

    // Wait for A→B connection
    let connected = wait_for_event(&mut rx_a, |e| {
        if let NetworkEvent::PeerConnected(pid) = e {
            Some(*pid)
        } else {
            None
        }
    })
    .expect("Swarm A should see B connect within 10 s");
    assert_eq!(connected, peer_b, "A should see B connected");
    tracing::info!("Swarm A confirmed connection from B");

    // Give gossipsub time to exchange subscriptions and stabilise the mesh.
    std::thread::sleep(Duration::from_millis(6000));

    // --- Gossipsub messaging ---
    let topic = p2p::get_game_topic();
    swarm_b.publish(topic, Message::Ping { peer_id: peer_b });

    let raw = wait_for_event(&mut rx_a, |e| {
        if let NetworkEvent::Message(pid, _topic, data) = e {
            if *pid == peer_b {
                Some(data.clone())
            } else {
                None
            }
        } else {
            None
        }
    })
    .expect("Swarm A should receive a message from B within 10 s");

    let decoded: Message = bincode::deserialize(&raw)?;
    assert!(
        matches!(decoded, Message::Ping { .. }),
        "Expected Message::Ping, got {decoded:?}"
    );

    tracing::info!("SUCCESS: TCP dial + gossipsub messaging works between two local swarms");
    Ok(())
}

fn wait_for_event<F, T>(rx: &mut tokio::sync::mpsc::Receiver<NetworkEvent>, f: F) -> Option<T>
where
    F: Fn(&NetworkEvent) -> Option<T>,
{
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match rx.try_recv() {
            Ok(event) => {
                if let Some(result) = f(&event) {
                    return Some(result);
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return None,
        }
    }
    None
}
