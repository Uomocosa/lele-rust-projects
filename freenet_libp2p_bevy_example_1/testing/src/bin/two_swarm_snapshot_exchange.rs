use std::time::Duration;

use freenet_libp2p_bevy_example_1_lib::p2p;
use libp2p::PeerId;
use libp2p::identity::Keypair;

fn fail<T>(msg: &str) -> Result<T, Box<dyn std::error::Error>> {
    Err(msg.into())
}

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

async fn wait_snapshot(rx: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event>) -> p2p::Snapshot {
    loop {
        if let Some(p2p::Event::IncomingSnapshot { snapshot, .. }) = rx.recv().await {
            return snapshot;
        }
    }
}

/// Two separate libp2p swarms built by the same `p2p::run` loop exchange connection
/// confirmation and snapshot messages, proving the request/response + behaviour wiring works
/// end-to-end. This is a `[[bin]]` (not a `#[tokio::test]`) so cargo emits it at a stable,
/// un-hashed path, letting a Windows Firewall rule keyed to that path stay valid across builds
/// (a hashed `deps/` test binary would re-prompt on every dependency change).
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cmd_tx_a, cmd_rx_a) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
    let (event_tx_a, mut event_rx_a) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
    let (cmd_tx_b, cmd_rx_b) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
    let (event_tx_b, mut event_rx_b) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();

    let task_a = tokio::spawn(p2p::run(cmd_rx_a, event_tx_a, Keypair::generate_ed25519()));
    let task_b = tokio::spawn(p2p::run(cmd_rx_b, event_tx_b, Keypair::generate_ed25519()));

    let a_ready = tokio::time::timeout(Duration::from_secs(10), wait_ready(&mut event_rx_a)).await;
    if a_ready.is_err() {
        return fail("swarm A never became ready");
    }
    let (a_peer, a_addrs) = a_ready.unwrap();

    let b_ready = tokio::time::timeout(Duration::from_secs(10), wait_ready(&mut event_rx_b)).await;
    if b_ready.is_err() {
        return fail("swarm B never became ready");
    }
    let (b_peer, b_addrs) = b_ready.unwrap();

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
    if a_saw_b.is_err() {
        return fail("A never connected to B");
    }
    let b_saw_a =
        tokio::time::timeout(Duration::from_secs(10), wait_connected(&mut event_rx_b)).await;
    if b_saw_a.is_err() {
        return fail("B never connected to A");
    }

    let b_snapshot = p2p::Snapshot {
        player_id: 2,
        x: 5.0,
        y: 6.0,
        vx: 0.5,
        vy: 0.25,
        tick: 1,
        sent_at_ms: 1,
    };
    cmd_tx_b
        .send(p2p::Command::SendSnapshot {
            peer_id: a_peer.clone(),
            snapshot: b_snapshot,
        })
        .ok();

    let a_saw_b_snapshot =
        tokio::time::timeout(Duration::from_secs(10), wait_snapshot(&mut event_rx_a)).await;
    if a_saw_b_snapshot.is_err() {
        return fail("A never received B's snapshot");
    }
    let a_recv = a_saw_b_snapshot.unwrap();
    if a_recv != b_snapshot {
        return fail("A received a different snapshot than B sent");
    }

    let a_snapshot = p2p::Snapshot {
        player_id: 1,
        x: 1.0,
        y: 2.0,
        vx: 0.1,
        vy: 0.2,
        tick: 1,
        sent_at_ms: 2,
    };
    cmd_tx_a
        .send(p2p::Command::SendSnapshot {
            peer_id: b_peer.clone(),
            snapshot: a_snapshot,
        })
        .ok();

    let b_saw_a_snapshot =
        tokio::time::timeout(Duration::from_secs(10), wait_snapshot(&mut event_rx_b)).await;
    if b_saw_a_snapshot.is_err() {
        return fail("B never received A's snapshot");
    }
    let b_recv = b_saw_a_snapshot.unwrap();
    if b_recv != a_snapshot {
        return fail("B received a different snapshot than A sent");
    }

    let a_saw_b_reply =
        tokio::time::timeout(Duration::from_secs(10), wait_snapshot(&mut event_rx_a)).await;
    if a_saw_b_reply.is_err() {
        return fail("A never received B's reply snapshot");
    }
    let a_reply = a_saw_b_reply.unwrap();
    if a_reply != b_snapshot {
        return fail("A received a different reply snapshot than expected");
    }

    task_a.abort();
    task_b.abort();
    let _ = b_addrs;
    Ok(())
}

// no test_usage necessary
