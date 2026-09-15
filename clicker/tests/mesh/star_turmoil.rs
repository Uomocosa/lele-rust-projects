use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use bevy::prelude::*;
use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{p2p, roster};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PORT: u16 = 17381;
const HANDSHAKE_ITERS: u32 = 300;
const CONVERGE_ITERS: u32 = 2000;
const STEP_SLEEP: Duration = Duration::from_millis(5);
const MAX_FRAME: usize = 4_000_000;

type Results = Rc<RefCell<Vec<Option<testing::MeshCount>>>>;
type Envelope = (String, u64, p2p::Event<clicker::CursorMsg>);

fn counts_of(app: &mut App) -> testing::MeshCount {
    let mut out = testing::MeshCount::default();
    let mut query = app
        .world_mut()
        .query::<(&clicker::PlayerNo, &clicker::ClickCounter)>();
    for (player, counter) in query.iter(app.world()) {
        if **player == 1 {
            out.p1 = **counter;
        } else if **player == 2 {
            out.p2 = **counter;
        } else if **player == 3 {
            out.p3 = **counter;
        }
    }
    out.global = **app.world().resource::<clicker::GlobalCounter>();
    out
}

fn labeled(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&clicker::PlayerNo>();
    query.iter(app.world()).count()
}

fn link_app(app: &mut App, name: &str) {
    let entries = [
        ([1u8; 32], "peer-1"),
        ([2u8; 32], "peer-2"),
        ([3u8; 32], "peer-3"),
    ];
    {
        let mut roster = app.world_mut().resource_mut::<roster::Roster>();
        for (id, peer) in entries {
            if peer != name {
                roster.add_entry("alpha".to_string(), id, peer.to_string());
            }
        }
    }
    {
        let mut events = app
            .world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>();
        for (_, peer) in entries {
            if peer != name {
                events.push(p2p::Event::PeerConnected(peer.to_string()));
            }
        }
    }
}

async fn write_event(
    stream: &mut turmoil::net::TcpStream,
    envelope: &(String, u64, p2p::Event<clicker::CursorMsg>),
) -> std::io::Result<()> {
    let bytes = bincode::serialize(envelope)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let len = u32::try_from(bytes.len())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    stream.write_u32(len).await?;
    stream.write_all(&bytes).await
}

async fn read_event(
    stream: &mut turmoil::net::TcpStream,
) -> std::io::Result<(String, u64, p2p::Event<clicker::CursorMsg>)> {
    let len = stream.read_u32().await?;
    let len = usize::try_from(len)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if len > MAX_FRAME {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    bincode::deserialize(&buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

async fn pump(
    app: &mut App,
    name: &str,
    outbound: &mut HashMap<String, turmoil::net::TcpStream>,
    inbox: &mut tokio::sync::mpsc::UnboundedReceiver<Envelope>,
    seen: &mut HashSet<(String, u64)>,
    seq: &mut u64,
) {
    app.update();
    let cmds = app
        .world_mut()
        .resource_mut::<p2p::Commands<clicker::CursorMsg>>()
        .take_all();
    for cmd in cmds {
        let (targets, event) = match cmd {
            p2p::Command::Send { peer_id, payload } => (
                vec![peer_id],
                p2p::Event::Message {
                    from: name.to_string(),
                    payload,
                },
            ),
            p2p::Command::PutHistory { lobby, chunk, data } => (
                outbound.keys().cloned().collect(),
                p2p::Event::HistoryChunk { lobby, chunk, data },
            ),
            p2p::Command::Publish { topic, data } => (
                outbound.keys().cloned().collect(),
                p2p::Event::Gossip {
                    topic,
                    from: name.to_string(),
                    data,
                },
            ),
            _ => continue,
        };
        let id = *seq;
        *seq = seq.saturating_add(1);
        let envelope = (name.to_string(), id, event);
        for target in targets {
            if let Some(stream) = outbound.get_mut(&target) {
                write_event(stream, &envelope).await.ok();
            }
        }
    }
    let mut pending = Vec::new();
    while let Ok((origin, id, event)) = inbox.try_recv() {
        if matches!(event, p2p::Event::Gossip { .. }) {
            if !seen.insert((origin.clone(), id)) {
                continue;
            }
            let envelope = (origin, id, event.clone());
            for stream in outbound.values_mut() {
                write_event(stream, &envelope).await.ok();
            }
        }
        pending.push(event);
    }
    if !pending.is_empty() {
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .extend(pending);
    }
}

async fn read_loop(
    mut stream: turmoil::net::TcpStream,
    inbox: tokio::sync::mpsc::UnboundedSender<Envelope>,
) {
    loop {
        match read_event(&mut stream).await {
            Ok(envelope) => {
                if inbox.send(envelope).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn run_peer(
    name: &'static str,
    own: u64,
    clicks: u32,
    slot: usize,
    dial: &'static [&'static str],
    accept: usize,
    results: Results,
) -> impl std::future::Future<Output = turmoil::Result> + 'static {
    async move {
        let mut app = testing::fixture(own, "alpha");
        link_app(&mut app, name);
        let listener = turmoil::net::TcpListener::bind(("0.0.0.0", PORT)).await?;
        let mut outbound = HashMap::new();
        let mut inbound = Vec::new();
        let connect_all = async {
            for peer in dial {
                let mut attempts = 0u32;
                loop {
                    match turmoil::net::TcpStream::connect((*peer, PORT)).await {
                        Ok(stream) => {
                            outbound.insert(peer.to_string(), stream);
                            break;
                        }
                        Err(_) => {
                            attempts = attempts.saturating_add(1);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        };
        let accept_all = async {
            for _ in 0..accept {
                let accept = tokio::time::timeout(Duration::from_secs(30), listener.accept())
                    .await
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::TimedOut, e))??;
                inbound.push(accept.0);
            }
            Ok::<(), std::io::Error>(())
        };
        let ((), accept_ok) = tokio::join!(connect_all, accept_all);
        accept_ok?;
        let (inbox_tx, mut inbox_rx) = tokio::sync::mpsc::unbounded_channel::<Envelope>();
        for stream in inbound {
            tokio::spawn(read_loop(stream, inbox_tx.clone()));
        }
        drop(inbox_tx);
        let mut seen = HashSet::new();
        let mut seq = 0u64;
        for _ in 0..HANDSHAKE_ITERS {
            pump(
                &mut app,
                name,
                &mut outbound,
                &mut inbox_rx,
                &mut seen,
                &mut seq,
            )
            .await;
            if labeled(&mut app) >= 3 {
                break;
            }
            tokio::time::sleep(STEP_SLEEP).await;
        }
        testing::click_times(&mut app, clicks);
        let want = testing::MeshCount::wanted(1, 5, 17);
        for _ in 0..CONVERGE_ITERS {
            pump(
                &mut app,
                name,
                &mut outbound,
                &mut inbox_rx,
                &mut seen,
                &mut seq,
            )
            .await;
            if counts_of(&mut app) == want {
                break;
            }
            tokio::time::sleep(STEP_SLEEP).await;
        }
        results.borrow_mut()[slot] = Some(counts_of(&mut app));
        Ok(())
    }
}

#[test]
fn star_leaf_to_leaf_heals_via_center() {
    let results: Results = Rc::new(RefCell::new(vec![None, None, None]));
    let mut sim = turmoil::Builder::new()
        .rng_seed(7)
        .min_message_latency(Duration::from_millis(5))
        .simulation_duration(Duration::from_secs(600))
        .build();
    let lanes: [(&'static str, u64, u32, &'static [&'static str], usize); 3] = [
        ("peer-1", 1, 1, &["peer-2", "peer-3"], 2),
        ("peer-2", 2, 5, &["peer-1"], 1),
        ("peer-3", 3, 17, &["peer-1"], 1),
    ];
    for (slot, (name, own, clicks, dial, accept)) in lanes.into_iter().enumerate() {
        let seen = results.clone();
        sim.host(name, move || {
            run_peer(name, own, clicks, slot, dial, accept, seen.clone())
        });
    }
    let seen = results.clone();
    sim.client("client", async move {
        let settled = tokio::time::timeout(Duration::from_secs(180), async {
            loop {
                if seen.borrow().iter().all(|r| r.is_some()) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await;
        if settled.is_err() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::TimedOut, "peers never reported").into(),
            );
        }
        Ok(())
    });
    sim.run().expect("turmoil sim");
    let want = testing::MeshCount::wanted(1, 5, 17);
    let results = results.borrow();
    for (slot, got) in results.iter().enumerate() {
        assert_eq!(*got, Some(want), "peer-{} diverged", slot.saturating_add(1));
    }
}
