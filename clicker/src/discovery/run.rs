use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use freenet_libp2p_bevy_plugin::p2p;
use tracing::{info, warn};

use crate::clicker;
use crate::constants;
use crate::discovery;

const READY_TIMEOUT_SECS: u64 = 120;
const TICK_SECS: u64 = 1;
const ANNOUNCE_SECS: u64 = 30;
const WARMUP_SECS: u64 = 15;
const WARMUP_ANNOUNCE_SECS: u64 = 5;
const DISCOVERY_SECS: u64 = 300;
const DIRECTORY_TICK_SECS: u64 = 5;

type ConnectedMap = std::collections::HashMap<String, u32>;
type StaggerMap =
    std::collections::HashMap<String, (std::collections::VecDeque<String>, std::time::Instant)>;

struct RunContext {
    cmd_tx: tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    roster: discovery::Roster,
    directory: discovery::Directory,
    room: String,
    room_params: Vec<u8>,
    peer_id: String,
    links: tokio::sync::mpsc::UnboundedReceiver<(String, bool)>,
    lobby_events: tokio::sync::mpsc::UnboundedReceiver<p2p::Event<clicker::CursorMsg>>,
    observed: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    last_addrs: Vec<String>,
    last_announce: std::time::Instant,
    last_gossip: std::time::Instant,
    last_redial: std::time::Instant,
    last_directory: std::time::Instant,
    last_directory_bridge: Option<std::time::Instant>,
    started: std::time::Instant,
    connected: ConnectedMap,
    attempted: std::collections::HashMap<String, std::time::Instant>,
    staggers: StaggerMap,
    transport: p2p::TransportMode,
    directory_tx: tokio::sync::mpsc::UnboundedSender<discovery::DirectoryState>,
}

pub async fn run(mut config: discovery::RunConfig) {
    let Some((peer_id, addrs)) = wait_ready(&mut config.ready).await else {
        warn!(target: "clicker", "discovery: no libp2p ready signal, discovery disabled");
        return;
    };
    let addrs = dialable(addrs);
    let (_node_guard, ws_port) = match bootstrap_node().await {
        Ok(node) => node,
        Err(e) => {
            warn!(target: "clicker", error = %e, "discovery: node bootstrap failed");
            return;
        }
    };
    let mut directory = loop {
        match discovery::connect_directory(
            "127.0.0.1",
            ws_port,
            clicker::contract_wasm(),
            &discovery::dir_params(&config.namespace),
        )
        .await
        {
            Ok(directory) => break directory,
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: directory connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    };
    info!(target: "clicker", key = %directory.contract_key, "discovery: directory connected");
    let Some((room, room_params)) =
        resolve_room(&mut config, &mut directory, &peer_id, &addrs).await
    else {
        warn!(target: "clicker", "discovery: no room resolved, discovery disabled");
        return;
    };
    info!(target: "clicker", room = %room, "discovery: room resolved");
    config.room_tx.send_replace(Some(room.clone()));
    let roster = loop {
        match discovery::connect_roster(
            "127.0.0.1",
            ws_port,
            clicker::contract_wasm(),
            &room_params,
            config.own,
            &peer_id,
            &addrs,
        )
        .await
        {
            Ok(roster) => break roster,
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: roster connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    };
    info!(target: "clicker", key = %roster.contract_key, own = *config.own, "discovery: roster connected");
    let connected: ConnectedMap = std::collections::HashMap::new();
    let mut staggers: StaggerMap = std::collections::HashMap::new();
    dial_known(
        &config.cmd_tx,
        &connected,
        &mut staggers,
        &roster.slots,
        config.own,
        config.transport,
    );
    if roster.announce().is_err() {
        warn!(target: "clicker", "discovery: initial announce failed");
    }
    let now = std::time::Instant::now();
    let mut ctx = RunContext {
        cmd_tx: config.cmd_tx,
        roster,
        directory,
        room,
        room_params,
        peer_id,
        links: config.links,
        lobby_events: config.lobby_events,
        observed: config.observed,
        last_addrs: addrs,
        last_announce: now,
        last_gossip: now,
        last_redial: now,
        last_directory: now,
        last_directory_bridge: None,
        started: now,
        connected,
        attempted: std::collections::HashMap::new(),
        staggers,
        transport: config.transport,
        directory_tx: config.directory_tx,
    };
    drive_roster(&mut ctx).await;
}

// needed helper: counts one open connection per peer so sub-connection
// churn never reads as a full drop (the swarm reports per-connection,
// while redial must only fire when the last connection closes)
fn count_link(ctx: &mut RunContext, peer: String) {
    ctx.connected
        .entry(peer)
        .and_modify(|count| *count = count.saturating_add(1))
        .or_insert(1);
}

// needed helper: drops one open connection; the peer leaves only at zero
fn decrement_link(ctx: &mut RunContext, peer: &str) {
    let drained = ctx.connected.get_mut(peer).is_some_and(|count| {
        *count = count.saturating_sub(1);
        *count == 0
    });
    if drained {
        ctx.connected.remove(peer);
    }
}

// needed helper: drives the connected roster steady-state loop
async fn drive_roster(ctx: &mut RunContext) {
    let roster_topic = format!("clicker/{}/roster", ctx.room);
    loop {
        while let Ok((peer, up)) = ctx.links.try_recv() {
            if up {
                count_link(ctx, peer);
            } else {
                decrement_link(ctx, &peer);
            }
        }
        drain_lobby_events(
            &ctx.cmd_tx,
            &mut ctx.lobby_events,
            &mut ctx.attempted,
            &ctx.peer_id,
            &ctx.connected,
            &mut ctx.staggers,
            ctx.transport,
        );
        match ctx.roster.poll().await {
            Ok(fresh) => {
                for entry in fresh {
                    dial_tiebreak(
                        &ctx.cmd_tx,
                        &mut ctx.attempted,
                        &ctx.connected,
                        &mut ctx.staggers,
                        &ctx.peer_id,
                        ctx.transport,
                        &entry,
                    );
                }
            }
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: poll failed");
            }
        }
        if ctx.last_directory.elapsed().as_secs() >= DIRECTORY_TICK_SECS {
            ctx.last_directory = std::time::Instant::now();
            refresh_directory(ctx).await;
        }
        if ctx.last_gossip.elapsed().as_secs_f64() >= discovery::ROSTER_HEARTBEAT_SECS {
            ctx.last_gossip = std::time::Instant::now();
            publish_slots(&ctx.cmd_tx, &roster_topic, &ctx.roster.slots);
        }
        if ctx.last_redial.elapsed().as_secs() >= discovery::REDIAL_SECS {
            ctx.last_redial = std::time::Instant::now();
            redial_missing(ctx);
        }
        fire_due_staggers(&ctx.cmd_tx, &ctx.connected, &mut ctx.staggers);
        if ctx.observed.has_changed().unwrap_or(false) {
            refresh_observed(ctx);
        }
        let interval = if ctx.started.elapsed().as_secs() < WARMUP_SECS {
            WARMUP_ANNOUNCE_SECS
        } else {
            ANNOUNCE_SECS
        };
        if ctx.last_announce.elapsed().as_secs() >= interval {
            ctx.last_announce = std::time::Instant::now();
            if ctx.roster.announce().is_err() {
                warn!(target: "clicker", "discovery: announce failed");
            }
        }
        if ctx.roster.bridge_tick(std::time::Instant::now()).is_err() {
            warn!(target: "clicker", "discovery: bridge failed");
        }
        tokio::time::sleep(Duration::from_secs(TICK_SECS)).await;
    }
}

// needed helper: polls the directory, dials new hints, refreshes our room entry
async fn refresh_directory(ctx: &mut RunContext) {
    match ctx.directory.poll().await {
        Ok(slots) => {
            ctx.directory_tx.send(slots.clone()).ok();
            for hint in directory_hints(&slots) {
                dial_hint(
                    &ctx.cmd_tx,
                    &mut ctx.attempted,
                    &ctx.connected,
                    &mut ctx.staggers,
                    &ctx.peer_id,
                    ctx.transport,
                    &hint,
                );
            }
        }
        Err(e) => {
            warn!(target: "clicker", error = %e, "discovery: directory poll failed");
        }
    }
    if ctx
        .directory
        .publish_room(&ctx.room, &ctx.room_params, &ctx.peer_id, &ctx.last_addrs)
        .is_err()
    {
        warn!(target: "clicker", "discovery: directory refresh failed");
    }
    if ctx
        .directory
        .bridge_tick(&mut ctx.last_directory_bridge, std::time::Instant::now())
        .is_err()
    {
        warn!(target: "clicker", "discovery: directory bridge failed");
    }
}

// needed helper: redials known-but-disconnected roster peers
fn redial_missing(ctx: &mut RunContext) {
    for (id, entry) in &ctx.roster.slots {
        if *id == ctx.roster.own || ctx.connected.contains_key(&entry.peer_id) {
            continue;
        }
        dial_tiebreak(
            &ctx.cmd_tx,
            &mut ctx.attempted,
            &ctx.connected,
            &mut ctx.staggers,
            &ctx.peer_id,
            ctx.transport,
            entry,
        );
    }
}

// needed helper: refreshes roster addrs when the observed address changes
fn refresh_observed(ctx: &mut RunContext) {
    let seen = ctx.observed.borrow_and_update().clone().unwrap_or_default();
    let dialable_seen = dialable(seen);
    if dialable_seen.is_empty() || dialable_seen == ctx.last_addrs {
        return;
    }
    ctx.last_addrs.clone_from(&dialable_seen);
    ctx.roster.refresh_addrs(dialable_seen);
    ctx.last_announce = std::time::Instant::now();
    if ctx.roster.announce().is_err() {
        warn!(target: "clicker", "discovery: re-announce failed");
    }
}

// needed helper: resolves the room via request publish, directory list, or auto-join
async fn resolve_room(
    config: &mut discovery::RunConfig,
    directory: &mut discovery::Directory,
    peer_id: &str,
    addrs: &[String],
) -> Option<(String, Vec<u8>)> {
    if let Some(room) = config.lobby.as_deref() {
        let params =
            discovery::resolve_params(&config.namespace, room, config.params_override.as_deref());
        parallel_probe(
            &config.cmd_tx,
            directory,
            &mut std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            &mut std::collections::HashMap::new(),
            config.transport,
            peer_id,
        )
        .await;
        if directory
            .publish_room(room, &params, peer_id, addrs)
            .is_err()
        {
            warn!(target: "clicker", "discovery: room publish failed");
        }
        return Some((room.to_string(), params));
    }
    let deadline = std::time::Instant::now().checked_add(Duration::from_secs(DISCOVERY_SECS))?;
    let mut attempted: std::collections::HashMap<String, std::time::Instant> =
        std::collections::HashMap::new();
    let connected: ConnectedMap = std::collections::HashMap::new();
    let mut staggers: StaggerMap = std::collections::HashMap::new();
    loop {
        if let Ok(room) = config.room_requests.try_recv() {
            let params = discovery::resolve_params(
                &config.namespace,
                &room,
                config.params_override.as_deref(),
            );
            if directory
                .publish_room(&room, &params, peer_id, addrs)
                .is_err()
            {
                warn!(target: "clicker", "discovery: room publish failed");
            }
            return Some((room, params));
        }
        parallel_probe(
            &config.cmd_tx,
            directory,
            &mut attempted,
            &connected,
            &mut staggers,
            config.transport,
            peer_id,
        )
        .await;
        fire_due_staggers(&config.cmd_tx, &connected, &mut staggers);
        match directory.poll().await {
            Ok(slots) => {
                config.directory_tx.send(slots.clone()).ok();
                if let Some((room, entry)) = discovery::pick_room(&slots, config.since_secs) {
                    let params = room_params(
                        &config.namespace,
                        &room,
                        config.params_override.as_deref(),
                        entry,
                    );
                    return Some((room, params));
                }
            }
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: room poll failed");
            }
        }
        if std::time::Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(Duration::from_secs(DIRECTORY_TICK_SECS)).await;
    }
}

// needed helper: picks join params from the directory entry or recomputes them
fn room_params(
    namespace: &str,
    room: &str,
    params_override: Option<&str>,
    entry: discovery::DirectoryEntry,
) -> Vec<u8> {
    if entry.params.is_empty() {
        discovery::resolve_params(namespace, room, params_override)
    } else {
        entry.params
    }
}

// needed helper: runs one parallel freenet+libp2p probe round and bootstraps hints
async fn parallel_probe(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    directory: &mut discovery::Directory,
    attempted: &mut std::collections::HashMap<String, std::time::Instant>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    mode: p2p::TransportMode,
    peer_id: &str,
) {
    match directory.poll().await {
        Ok(slots) => {
            for hint in directory_hints(&slots) {
                dial_hint(cmd_tx, attempted, connected, staggers, peer_id, mode, &hint);
            }
        }
        Err(e) => {
            warn!(target: "clicker", error = %e, "discovery: probe poll failed");
        }
    }
    cmd_tx
        .send(p2p::Command::FindLobby {
            lobby: directory_lobby(),
        })
        .ok();
}

// needed helper: libp2p directory topic used for provider discovery
fn directory_lobby() -> String {
    discovery::DIRECTORY_LOBBY.to_string()
}

// needed helper: converts directory entries into dialable peer hints
fn directory_hints(slots: &discovery::DirectoryState) -> Vec<discovery::PeerHint> {
    let hints: Vec<discovery::PeerHint> = slots
        .values()
        .map(|entry| discovery::PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            updated_at: entry.updated_at,
        })
        .collect();
    discovery::merge_peer_hints(hints)
}

// needed helper: drains libp2p lobby/provider/gossip events into dials
fn drain_lobby_events(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    lobby_events: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event<clicker::CursorMsg>>,
    attempted: &mut std::collections::HashMap<String, std::time::Instant>,
    peer_id: &str,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    mode: p2p::TransportMode,
) {
    while let Ok(event) = lobby_events.try_recv() {
        match event {
            p2p::Event::LobbyProviders { peers, .. } => {
                for peer in peers {
                    dial_hint(
                        cmd_tx,
                        attempted,
                        connected,
                        staggers,
                        peer_id,
                        mode,
                        &discovery::PeerHint {
                            peer_id: peer,
                            addrs: Vec::new(),
                            updated_at: epoch_secs(),
                        },
                    );
                }
            }
            p2p::Event::Gossip { topic, from, data } => {
                if !is_roster_topic(&topic) {
                    continue;
                }
                let slots: discovery::RosterState = bincode::deserialize(&data).unwrap_or_default();
                for entry in slots.values() {
                    if entry.peer_id == from || entry.peer_id.is_empty() {
                        continue;
                    }
                    dial_hint(
                        cmd_tx,
                        attempted,
                        connected,
                        staggers,
                        peer_id,
                        mode,
                        &discovery::PeerHint {
                            peer_id: entry.peer_id.clone(),
                            addrs: entry.addrs.clone(),
                            updated_at: entry.updated_at,
                        },
                    );
                }
            }
            p2p::Event::PeerConnected(peer) => {
                attempted.insert(peer, std::time::Instant::now());
            }
            _ => {}
        }
    }
}

// needed helper: checks whether a gossip topic is a roster mirror topic
fn is_roster_topic(topic: &str) -> bool {
    topic.starts_with("clicker/") && topic.ends_with("/roster")
}

// needed helper: waits for the libp2p ready signal carrying our listen addrs
async fn wait_ready(
    ready: &mut tokio::sync::watch::Receiver<Option<(String, Vec<String>)>>,
) -> Option<(String, Vec<String>)> {
    let deadline =
        tokio::time::Instant::now().checked_add(Duration::from_secs(READY_TIMEOUT_SECS))?;
    loop {
        let value = ready.borrow().clone();
        if let Some(info) = value {
            return Some(info);
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return None;
        }
        if tokio::time::timeout(remaining, ready.changed())
            .await
            .is_err()
        {
            return ready.borrow().clone();
        }
    }
}

// needed helper: drops wildcard addrs that no peer can dial back
fn dialable(addrs: Vec<String>) -> Vec<String> {
    addrs
        .into_iter()
        .filter(|a| !a.contains("0.0.0.0"))
        .collect()
}

// needed helper: starts an embedded client-peer node on the real mainnet with free ports
async fn bootstrap_node() -> Result<(tempfile::TempDir, u16), discovery::Error> {
    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener
        .local_addr()
        .map_err(discovery::Error::from)?
        .port();
    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener)
        .await
        .map_err(|e| discovery::Error::Node(e.to_string()))?;
    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            network_port: Some(free_udp_port()?),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = args
        .build()
        .await
        .map_err(|e| discovery::Error::Node(e.to_string()))?;
    let node_config = NodeConfig::new(config)
        .await
        .map_err(|e| discovery::Error::Node(e.to_string()))?;
    let node = node_config
        .build(clients)
        .await
        .map_err(|e| discovery::Error::Node(e.to_string()))?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            warn!(target: "clicker", error = %e, "embedded freenet node exited");
        }
    });
    info!(target: "clicker", ws_port, "embedded freenet node started");
    Ok((tmp, ws_port))
}

// needed helper: allocates a free UDP port so parallel instances never collide
fn free_udp_port() -> Result<u16, discovery::Error> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = socket.local_addr().map_err(discovery::Error::from)?.port();
    Ok(port)
}

// needed helper: dials foreign roster entries already present at connect time
fn dial_known(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    slots: &discovery::RosterState,
    own: discovery::PlayerId,
    mode: p2p::TransportMode,
) {
    for (id, entry) in slots {
        if *id == own {
            continue;
        }
        dial_preferred(cmd_tx, connected, staggers, mode, entry);
    }
}

// needed helper: dials fresh entries with a deterministic tie-break so two
// peers never dial each other simultaneously (simultaneous multi-addr dials
// collapse the link within milliseconds); the higher id waits for inbound
// and only dials itself after two silent redial periods
fn dial_tiebreak(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    attempted: &mut std::collections::HashMap<String, std::time::Instant>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    own_peer_id: &str,
    mode: p2p::TransportMode,
    entry: &discovery::PeerEntry,
) {
    if entry.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let age = attempted.get(&entry.peer_id).and_then(|seen| {
        std::time::Instant::now()
            .checked_duration_since(*seen)
            .map(|d| d.as_secs())
    });
    if age.is_some_and(|seen_secs| seen_secs < discovery::REDIAL_SECS) {
        return;
    }
    let decision = discovery::decide_dial(own_peer_id, &entry.peer_id, age);
    apply_decision(
        cmd_tx, attempted, connected, staggers, mode, entry, decision,
    );
}

// needed helper: applies a dial decision for one peer entry
fn apply_decision(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    attempted: &mut std::collections::HashMap<String, std::time::Instant>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    mode: p2p::TransportMode,
    entry: &discovery::PeerEntry,
    decision: discovery::DialDecision,
) {
    match decision {
        discovery::DialDecision::Wait => {
            attempted
                .entry(entry.peer_id.clone())
                .or_insert_with(std::time::Instant::now);
        }
        discovery::DialDecision::Dial => {
            attempted.insert(entry.peer_id.clone(), std::time::Instant::now());
            dial_preferred(cmd_tx, connected, staggers, mode, entry);
        }
        discovery::DialDecision::ForceDial => {
            attempted.insert(entry.peer_id.clone(), std::time::Instant::now());
            dial_force_entry(cmd_tx, entry);
        }
    }
}

// needed helper: dials one bootstrapped peer hint discovered via directory or libp2p
fn dial_hint(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    attempted: &mut std::collections::HashMap<String, std::time::Instant>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    own_peer_id: &str,
    mode: p2p::TransportMode,
    hint: &discovery::PeerHint,
) {
    if hint.peer_id.is_empty() || hint.peer_id == own_peer_id {
        return;
    }
    if hint.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(hint.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let entry = discovery::PeerEntry {
        peer_id: hint.peer_id.clone(),
        addrs: hint.addrs.clone(),
        updated_at: hint.updated_at,
    };
    dial_tiebreak(
        cmd_tx,
        attempted,
        connected,
        staggers,
        own_peer_id,
        mode,
        &entry,
    );
}

// needed helper: dials the best-ranked addr now and staggers the rest
fn dial_preferred(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    mode: p2p::TransportMode,
    entry: &discovery::PeerEntry,
) {
    if connected.contains_key(&entry.peer_id) {
        staggers.remove(&entry.peer_id);
        return;
    }
    let ranked = discovery::rank_addrs(&with_loopback(&entry.addrs), mode);
    let mut queue: std::collections::VecDeque<String> = ranked.into();
    let Some(first) = queue.pop_front() else {
        return;
    };
    dial_addrs(cmd_tx, &entry.peer_id, &[first], &entry.addrs);
    if queue.is_empty() {
        staggers.remove(&entry.peer_id);
        return;
    }
    let due = std::time::Instant::now()
        .checked_add(Duration::from_secs(discovery::STAGGER_SECS))
        .unwrap_or_else(std::time::Instant::now);
    staggers.insert(entry.peer_id.clone(), (queue, due));
}

// needed helper: fires due staggered dials for peers still unreached
fn fire_due_staggers(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
) {
    for (peer, addr) in discovery::stagger_due(std::time::Instant::now(), connected, staggers) {
        info!(target: "clicker", peer = %peer, addr = %addr, "discovery: staggered dialing peer");
        cmd_tx
            .send(p2p::Command::DialForce {
                peer_id: peer,
                addrs: vec![addr],
            })
            .ok();
    }
}

// needed helper: sends one dial plus kad seeding for an addr set
fn dial_addrs(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    peer_id: &str,
    addrs: &[String],
    kad_addrs: &[String],
) {
    if addrs.is_empty() {
        return;
    }
    info!(target: "clicker", peer = %peer_id, addrs = ?addrs, "discovery: dialing peer");
    cmd_tx
        .send(p2p::Command::Dial {
            peer_id: peer_id.to_string(),
            addrs: addrs.to_vec(),
        })
        .ok();
    cmd_tx
        .send(p2p::Command::AddKadPeer {
            peer_id: peer_id.to_string(),
            addrs: kad_addrs.to_vec(),
        })
        .ok();
}

// needed helper: mirrors known roster slots onto the gossip roster topic
fn publish_slots(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    roster_topic: &str,
    slots: &discovery::RosterState,
) {
    let data = bincode::serialize(slots).unwrap_or_default();
    cmd_tx
        .send(p2p::Command::Publish {
            topic: roster_topic.to_string(),
            data,
        })
        .ok();
}

// needed helper: force-dials one peer entry past stuck swarm dial state
fn dial_force_entry(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    entry: &discovery::PeerEntry,
) {
    if entry.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    info!(target: "clicker", peer = %entry.peer_id, "discovery: force-dialing peer");
    cmd_tx
        .send(p2p::Command::DialForce {
            peer_id: entry.peer_id.clone(),
            addrs: with_loopback(&entry.addrs),
        })
        .ok();
}

// needed helper: seconds since the unix epoch for staleness checks
fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

// needed helper: appends 127.0.0.1 variants so same-host rigs survive hairpin-less NAT
fn with_loopback(addrs: &[String]) -> Vec<String> {
    let mut out: Vec<String> = addrs.to_vec();
    for addr in addrs {
        if let Some(rest) = addr.strip_prefix("/ip4/")
            && let Some(port) = rest.split("/tcp/").nth(1)
        {
            let ip = rest.split('/').next().unwrap_or_default();
            if ip != "127.0.0.1" && !ip.is_empty() && !port.is_empty() {
                out.push(format!("/ip4/127.0.0.1/tcp/{port}"));
            }
        }
    }
    out
}
// no test_usage necessary
