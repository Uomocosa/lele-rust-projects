use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use bevy::prelude::*;
use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const PORT: u16 = 17381;
pub const HANDSHAKE_ITERS: u32 = 300;
pub const CONVERGE_ITERS: u32 = 2000;
pub const STEP_SLEEP: Duration = Duration::from_millis(5);
pub const MAX_FRAME: usize = 4_000_000;

pub type Results = Rc<RefCell<Vec<Option<testing::MeshCount>>>>;
pub type Gone = Rc<RefCell<Vec<bool>>>;
pub type Done = Rc<RefCell<bool>>;
pub type Envelope = (String, u64, p2p::Event<clicker::CursorMsg>);

pub fn peer_key(name: &str) -> [u8; 32] {
    match name {
        "peer-1" => [1u8; 32],
        "peer-2" => [2u8; 32],
        _ => [3u8; 32],
    }
}

pub fn counts_of(app: &mut App) -> testing::MeshCount {
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

pub fn labeled(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&clicker::PlayerNo>();
    query.iter(app.world()).count()
}

pub fn link_app(app: &mut App, name: &str) {
    {
        let mut members = app.world_mut().resource_mut::<roster::Roster>();
        for peer in ["peer-1", "peer-2", "peer-3"] {
            if peer != name {
                members.add_entry("alpha".to_string(), peer_key(peer), peer.to_string());
            }
        }
    }
    {
        let mut events = app
            .world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>();
        for peer in ["peer-1", "peer-2", "peer-3"] {
            if peer != name {
                events.push(p2p::Event::PeerConnected(peer.to_string()));
            }
        }
    }
}

async fn write_event(
    stream: &mut turmoil::net::TcpStream,
    envelope: &Envelope,
) -> std::io::Result<()> {
    let bytes = bincode::serialize(envelope)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let len = u32::try_from(bytes.len())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    stream.write_u32(len).await?;
    stream.write_all(&bytes).await
}

async fn read_event(stream: &mut turmoil::net::TcpStream) -> std::io::Result<Envelope> {
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

pub async fn read_loop(
    mut stream: turmoil::net::TcpStream,
    inbox: tokio::sync::mpsc::UnboundedSender<Envelope>,
) {
    let mut origin: Option<String> = None;
    loop {
        match read_event(&mut stream).await {
            Ok((from, id, event)) => {
                origin = Some(from.clone());
                if inbox.send((from, id, event)).is_err() {
                    break;
                }
            }
            Err(_) => {
                if let Some(from) = origin.take() {
                    let event = p2p::Event::PeerDisconnected(from.clone());
                    inbox.send((from, 0, event)).ok();
                }
                break;
            }
        }
    }
}

fn prune(app: &mut App, peer: &str, dead: &mut HashSet<String>) {
    if dead.insert(peer.to_string()) {
        app.world_mut()
            .resource_mut::<roster::Roster>()
            .remove_entry("alpha", peer_key(peer));
    }
}

pub async fn pump(
    app: &mut App,
    name: &str,
    outbound: &mut HashMap<String, turmoil::net::TcpStream>,
    inbox: &mut tokio::sync::mpsc::UnboundedReceiver<Envelope>,
    seen: &mut HashSet<(String, u64)>,
    seq: &mut u64,
    dead: &mut HashSet<String>,
    last_sync: &mut HashMap<String, Duration>,
    link_down_after: Duration,
) {
    app.update();
    let now = turmoil::elapsed();
    let own = *app.world().resource::<net_id::NetworkId>();
    let peers: Vec<String> = outbound.keys().cloned().collect();
    for peer in peers {
        if dead.contains(&peer) {
            continue;
        }
        let last = last_sync.get(&peer).copied().unwrap_or(Duration::ZERO);
        if now.saturating_sub(last) >= link_down_after {
            last_sync.insert(peer.clone(), now);
            app.world_mut()
                .resource_mut::<p2p::Commands<clicker::CursorMsg>>()
                .push(p2p::Command::Send {
                    peer_id: peer,
                    payload: clicker::CursorMsg::SyncReq { requester: own },
                });
        }
    }
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
            if dead.contains(&target) {
                continue;
            }
            if let Some(stream) = outbound.get_mut(&target) {
                if write_event(stream, &envelope).await.is_err() {
                    prune(app, &target, dead);
                }
            }
        }
    }
    let mut pending = Vec::new();
    while let Ok((origin, id, event)) = inbox.try_recv() {
        if let p2p::Event::PeerDisconnected(peer) = &event {
            prune(app, peer, dead);
        }

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

pub struct UpLink {
    pub app: App,
    pub outbound: HashMap<String, turmoil::net::TcpStream>,
    pub inbox_rx: tokio::sync::mpsc::UnboundedReceiver<Envelope>,
    pub seen: HashSet<(String, u64)>,
    pub seq: u64,
    pub dead: HashSet<String>,
    pub last_sync: HashMap<String, Duration>,
    pub link_down_after: Duration,
}

pub async fn bring_up(
    name: &'static str,
    own: u64,
    dial: &[&'static str],
    accept: usize,
    link_down_after: Duration,
) -> turmoil::Result<UpLink> {
    let mut app = testing::fixture(own, "alpha");
    link_app(&mut app, name);
    let listener = turmoil::net::TcpListener::bind(("0.0.0.0", PORT)).await?;
    let mut outbound = HashMap::new();
    let mut inbound = Vec::new();
    let connect_all = async {
        for peer in dial {
            loop {
                match turmoil::net::TcpStream::connect((*peer, PORT)).await {
                    Ok(stream) => {
                        outbound.insert(peer.to_string(), stream);
                        break;
                    }
                    Err(_) => {
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
    let (inbox_tx, inbox_rx) = tokio::sync::mpsc::unbounded_channel::<Envelope>();
    for stream in inbound {
        tokio::spawn(read_loop(stream, inbox_tx.clone()));
    }
    drop(inbox_tx);
    Ok(UpLink {
        app,
        outbound,
        inbox_rx,
        seen: HashSet::new(),
        seq: 0,
        dead: HashSet::new(),
        last_sync: HashMap::new(),
        link_down_after,
    })
}

pub async fn handshake(link: &mut UpLink, name: &str) {
    for _ in 0..HANDSHAKE_ITERS {
        pump(
            &mut link.app,
            name,
            &mut link.outbound,
            &mut link.inbox_rx,
            &mut link.seen,
            &mut link.seq,
            &mut link.dead,
            &mut link.last_sync,
            link.link_down_after,
        )
        .await;
        if labeled(&mut link.app) >= 3 {
            break;
        }
        tokio::time::sleep(STEP_SLEEP).await;
    }
}

pub async fn converge_to(link: &mut UpLink, name: &str, want: testing::MeshCount) {
    for _ in 0..CONVERGE_ITERS {
        pump(
            &mut link.app,
            name,
            &mut link.outbound,
            &mut link.inbox_rx,
            &mut link.seen,
            &mut link.seq,
            &mut link.dead,
            &mut link.last_sync,
            link.link_down_after,
        )
        .await;
        if counts_of(&mut link.app) == want {
            break;
        }
        tokio::time::sleep(STEP_SLEEP).await;
    }
}

pub async fn settle(link: &mut UpLink, name: &str, iters: u32) {
    for _ in 0..iters {
        pump(
            &mut link.app,
            name,
            &mut link.outbound,
            &mut link.inbox_rx,
            &mut link.seen,
            &mut link.seq,
            &mut link.dead,
            &mut link.last_sync,
            link.link_down_after,
        )
        .await;
        tokio::time::sleep(STEP_SLEEP).await;
    }
}

pub async fn park_until_done(link: &mut UpLink, name: &str, done: &Done) {
    loop {
        if *done.borrow() {
            break;
        }
        pump(
            &mut link.app,
            name,
            &mut link.outbound,
            &mut link.inbox_rx,
            &mut link.seen,
            &mut link.seq,
            &mut link.dead,
            &mut link.last_sync,
            link.link_down_after,
        )
        .await;
        tokio::time::sleep(STEP_SLEEP).await;
    }
}

#[cfg(test)]
mod tests {
    use super::peer_key;

    #[test]
    fn test_usage() {
        assert_eq!(peer_key("peer-1"), [1u8; 32]);
        assert_eq!(peer_key("peer-2"), [2u8; 32]);
        assert_eq!(peer_key("peer-9"), [3u8; 32]);
    }
}
