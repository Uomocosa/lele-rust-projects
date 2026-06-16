use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use bevy_p2p_app::p2p::{self, Config, Message, NetworkEvent};

/// Run a single mDNS peer — faithfully simulates a real game client.
///
/// Starts Swarm with mDNS enabled, collects ALL discovered peers, dials all
/// of them, and exchanges Message::Ping with any connected peer — mirroring
/// real game behavior where mDNS discovers everyone on the LAN and the game
/// layer decides who to talk to. The orchestrator validates that the sibling
/// pair (A↔B) exists somewhere in the event stream.
pub fn run_mdns_peer() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .try_init();
    let config = Config::new().with_mdns(true).with_heartbeat(100);
    let (mut swarm, mut rx) = bevy_p2p_app::p2p::Swarm::new(config)?;
    let my_id = swarm.local_peer_id;

    let deadline = Instant::now() + Duration::from_secs(55);

    println!("Evt:READY:{}", my_id);

    let mut event_buffer: Vec<NetworkEvent> = Vec::new();

    drain_for_event_deadline(
        &mut rx,
        &mut event_buffer,
        |e| matches!(e, NetworkEvent::NewListenAddr(_)).then_some(()),
        deadline,
    )
    .ok_or_else(|| "Timeout waiting for listen address".to_string())?;

    // --- Phase 1: collect ALL mDNS discoveries ---
    let discovered = drain_events_deadline(
        &mut rx,
        &mut event_buffer,
        |e| {
            if let NetworkEvent::PeerDiscovered(pid, addr) = e {
                Some((*pid, addr.clone()))
            } else {
                None
            }
        },
        deadline,
        Duration::from_secs(10),
    );

    for (pid, _addr) in &discovered {
        println!("Evt:DISCOVERED:{}", pid);
    }

    // --- Phase 2: dial them all ---
    for (_, addr) in &discovered {
        println!("Evt:DIALING:{}", addr);
        // mDNS addresses include /p2p/<peerid> — strip it before dialing
        let mut dial_addr = addr.clone();
        dial_addr.pop();
        swarm.dial(dial_addr);
    }

    // --- Phase 3: collect ALL connections ---
    let connected = drain_events_deadline(
        &mut rx,
        &mut event_buffer,
        |e| {
            if let NetworkEvent::PeerConnected(pid) = e {
                Some(*pid)
            } else {
                None
            }
        },
        deadline,
        Duration::from_secs(10),
    );

    for pid in &connected {
        println!("Evt:CONNECTED:{}", pid);
    }

    // Give gossipsub time to exchange subscriptions and stabilise the mesh.
    thread::sleep(Duration::from_millis(6000));

    // --- Phase 4: publish Ping ---
    let topic = p2p::get_game_topic();
    println!("Evt:SENT_PING");
    swarm.publish(topic.clone(), Message::Ping { peer_id: my_id });

    // --- Phase 5: collect ALL incoming messages ---
    let remaining = deadline - Instant::now();
    let message_deadline = if remaining > Duration::ZERO {
        remaining
    } else {
        Duration::from_secs(5)
    };

    let messages = drain_events_deadline(
        &mut rx,
        &mut event_buffer,
        |e| {
            if let NetworkEvent::Message(pid, _topic, data) = e {
                Some((*pid, data.clone()))
            } else {
                None
            }
        },
        Instant::now() + message_deadline,
        message_deadline,
    );

    let mut got_ping = false;
    for (sender, raw) in &messages {
        match bincode::deserialize::<Message>(raw) {
            Ok(Message::Ping { .. }) => {
                println!("Evt:GOT_PING:{}", sender);
                got_ping = true;
            }
            Ok(other) => {
                tracing::warn!("Got unexpected message from {}: {:?}", sender, other);
            }
            Err(e) => {
                tracing::warn!("Failed to deserialize message from {}: {}", sender, e);
            }
        }
    }

    assert!(got_ping, "Must receive at least one Ping");
    println!("Evt:SUCCESS");
    Ok(())
}

/// Drains events matching a filter for up to `timeout`, returning all matches.
/// Non-matching events are stored in `buffer` so subsequent drain calls don't
/// lose them across phase boundaries.
pub fn drain_events_deadline<F, T>(
    rx: &mut tokio::sync::mpsc::Receiver<NetworkEvent>,
    buffer: &mut Vec<NetworkEvent>,
    f: F,
    overall_deadline: Instant,
    phase_timeout: Duration,
) -> Vec<T>
where
    F: Fn(&NetworkEvent) -> Option<T>,
{
    let mut results = Vec::new();
    let phase_deadline = Instant::now() + phase_timeout;

    // Check buffered events first (from previous phases that didn't match
    // that phase's filter but might match this one).
    let mut i = 0;
    while i < buffer.len() {
        if let Some(val) = f(&buffer[i]) {
            results.push(val);
            buffer.swap_remove(i);
        } else {
            i += 1;
        }
    }

    loop {
        if Instant::now() >= overall_deadline {
            break;
        }
        if Instant::now() >= phase_deadline {
            // One last try to drain anything buffered, then stop
            while let Ok(event) = rx.try_recv() {
                if let Some(val) = f(&event) {
                    results.push(val);
                } else {
                    buffer.push(event);
                }
            }
            break;
        }

        match rx.try_recv() {
            Ok(event) => {
                if let Some(val) = f(&event) {
                    results.push(val);
                } else {
                    buffer.push(event);
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }

    results
}

/// Wait for a single matching event, storing non-matching ones in `buffer`.
pub fn drain_for_event_deadline<F, T>(
    rx: &mut tokio::sync::mpsc::Receiver<NetworkEvent>,
    buffer: &mut Vec<NetworkEvent>,
    f: F,
    deadline: Instant,
) -> Option<T>
where
    F: Fn(&NetworkEvent) -> Option<T>,
{
    // Check buffered events first
    let mut i = 0;
    while i < buffer.len() {
        if let Some(result) = f(&buffer[i]) {
            buffer.swap_remove(i);
            return Some(result);
        }
        i += 1;
    }

    while Instant::now() < deadline {
        match rx.try_recv() {
            Ok(event) => {
                if let Some(result) = f(&event) {
                    return Some(result);
                }
                buffer.push(event);
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return None,
        }
    }
    None
}

/// Spawn a reader thread that reads lines from `reader` and sends them
/// through the mpsc channel, prefixed with `tag`.
#[allow(dead_code)]
pub fn spawn_stdout_reader(
    tag: &'static str,
    reader: impl Read + Send + 'static,
    tx: mpsc::Sender<String>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    let tagged = format!("{}:{}", tag, l);
                    if tx.send(tagged).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    })
}
