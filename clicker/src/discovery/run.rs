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
    expected_tx: tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    pex: discovery::HintStore,
    pex_asked: std::collections::HashMap<String, std::time::Instant>,
    known_rooms: std::collections::BTreeMap<String, discovery::DirectoryEntry>,
    pex_inbox: Vec<(String, clicker::CursorMsg)>,
    last_pex: std::time::Instant,
    room_requests: tokio::sync::mpsc::UnboundedReceiver<String>,
    room_tx: tokio::sync::watch::Sender<Option<String>>,
    ws_port: u16,
    own: discovery::PlayerId,
    namespace: String,
    params_override: Option<String>,
    pending_switch: Option<(String, u32)>,
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
    let mut directory = connect_directory_retry(ws_port, &config.namespace).await;
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
        let attempt = std::time::Instant::now();
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
            Ok(roster) => {
                info!(target: "clicker", room = %room, slots = roster.slots.len(), elapsed_ms = attempt.elapsed().as_millis(), "discovery: roster fetched");
                break roster;
            }
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: roster connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    };
    info!(target: "clicker", key = %roster.contract_key, own = *config.own, "discovery: roster connected");
    send_expected(&config.expected_tx, &roster.slots, config.own);
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
        room_requests: config.room_requests,
        room_tx: config.room_tx,
        ws_port,
        own: config.own,
        namespace: config.namespace,
        params_override: config.params_override,
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
        expected_tx: config.expected_tx,
        pending_switch: None,
        pex: discovery::HintStore::default(),
        pex_asked: std::collections::HashMap::new(),
        known_rooms: std::collections::BTreeMap::new(),
        pex_inbox: Vec::new(),
        last_pex: now,
    };
    drive_roster(&mut ctx).await;
}

// needed helper: connects the directory contract, retrying until the node answers
async fn connect_directory_retry(ws_port: u16, namespace: &str) -> discovery::Directory {
    loop {
        match discovery::connect_directory(
            "127.0.0.1",
            ws_port,
            clicker::contract_wasm(),
            &discovery::dir_params(namespace),
        )
        .await
        {
            Ok(directory) => return directory,
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: directory connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// needed helper: asks one fresh peer for its known peers and rooms
fn send_pex_ask(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    peer: &str,
) {
    cmd_tx
        .send(p2p::Command::Send {
            peer_id: peer.to_string(),
            payload: clicker::CursorMsg::PexAsk { want_rooms: true },
        })
        .ok();
}

// needed helper: re-asks connected peers whose world-view may have grown
fn reask_pex(ctx: &mut RunContext) {
    let now = std::time::Instant::now();
    let peers: Vec<String> = ctx.connected.keys().cloned().collect();
    for peer in peers {
        let due = ctx.pex_asked.get(&peer).is_none_or(|at| {
            now.checked_duration_since(*at)
                .is_none_or(|d| d.as_secs() >= discovery::PEX_INTERVAL_SECS)
        });
        if due {
            send_pex_ask(&ctx.cmd_tx, &peer);
            ctx.pex_asked.insert(peer, now);
        }
    }
    ctx.pex.prune(epoch_secs());
}

// needed helper: answers PEX asks and absorbs PEX responses into dials
fn drain_pex(ctx: &mut RunContext) {
    let inbox = std::mem::take(&mut ctx.pex_inbox);
    for (from, msg) in inbox {
        match msg {
            clicker::CursorMsg::PexAsk { .. } => {
                reply_pex(ctx, &from);
            }
            clicker::CursorMsg::PexResp { peers, rooms } => {
                absorb_pex_resp(ctx, &peers, &rooms);
            }
            _ => {}
        }
    }
}

// needed helper: replies with our roster slots plus known rooms
fn reply_pex(ctx: &RunContext, peer: &str) {
    let mut peers: Vec<discovery::PeerHint> = ctx
        .roster
        .slots
        .values()
        .map(|entry| discovery::PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: Vec::new(),
            updated_at: entry.updated_at,
        })
        .collect();
    for hint in ctx.pex.values() {
        peers.push(hint.clone());
    }
    let peers = discovery::merge_peer_hints(peers)
        .into_iter()
        .take(discovery::PEX_MAX_HINTS)
        .collect();
    let rooms: Vec<(String, discovery::DirectoryEntry)> = ctx
        .known_rooms
        .iter()
        .map(|(name, entry)| (name.clone(), entry.clone()))
        .chain([(
            ctx.room.clone(),
            discovery::DirectoryEntry {
                params: ctx.room_params.clone(),
                peer_id: ctx.peer_id.clone(),
                addrs: ctx.last_addrs.clone(),
                updated_at: epoch_secs(),
            },
        )])
        .take(discovery::PEX_MAX_ROOMS)
        .collect();
    ctx.cmd_tx
        .send(p2p::Command::Send {
            peer_id: peer.to_string(),
            payload: clicker::CursorMsg::PexResp { peers, rooms },
        })
        .ok();
}

// needed helper: merges PEX answers into the hint store, dials the unknown
fn absorb_pex_resp(
    ctx: &mut RunContext,
    peers: &[discovery::PeerHint],
    rooms: &[(String, discovery::DirectoryEntry)],
) {
    for hint in peers.iter().take(discovery::PEX_MAX_HINTS) {
        if hint.peer_id == ctx.peer_id {
            continue;
        }
        ctx.pex.insert(hint.clone());
        dial_hint(
            &ctx.cmd_tx,
            &mut ctx.attempted,
            &ctx.connected,
            &mut ctx.staggers,
            &ctx.peer_id,
            ctx.transport,
            hint,
        );
    }
    for (name, entry) in rooms.iter().take(discovery::PEX_MAX_ROOMS) {
        if name.is_empty() || entry.params.is_empty() {
            continue;
        }
        let keep = ctx
            .known_rooms
            .get(name)
            .is_none_or(|known| entry.updated_at >= known.updated_at);
        if keep {
            if !ctx.known_rooms.contains_key(name)
                && ctx.known_rooms.len() >= discovery::PEX_MAX_ROOMS
            {
                continue;
            }
            ctx.known_rooms.insert(name.clone(), entry.clone());
        }
    }
    send_merged_directory(ctx);
}

// needed helper: publishes the contract directory merged with PEX rooms
fn send_merged_directory(ctx: &RunContext) {
    let mut view = ctx.directory.slots.clone();
    for (name, entry) in &ctx.known_rooms {
        let keep = view
            .get(name)
            .is_none_or(|known| entry.updated_at >= known.updated_at);
        if keep {
            view.insert(name.clone(), entry.clone());
        }
    }
    ctx.directory_tx.send(view).ok();
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

// needed helper: reconnects roster and directory to a newly requested room
async fn switch_room(ctx: &mut RunContext, room: &str) -> bool {
    let params = discovery::resolve_params(&ctx.namespace, room, ctx.params_override.as_deref());
    let attempt = std::time::Instant::now();
    match discovery::connect_roster(
        "127.0.0.1",
        ctx.ws_port,
        clicker::contract_wasm(),
        &params,
        ctx.own,
        &ctx.peer_id,
        &ctx.last_addrs,
    )
    .await
    {
        Ok(roster) => {
            info!(target: "clicker", room = %room, slots = roster.slots.len(), elapsed_ms = attempt.elapsed().as_millis(), "discovery: roster fetched");
            ctx.roster = roster;
            send_expected(&ctx.expected_tx, &ctx.roster.slots, ctx.own);
            ctx.room = room.to_string();
            ctx.room_params = params;
            ctx.room_tx.send_replace(Some(room.to_string()));
            ctx.attempted.clear();
            ctx.staggers.clear();
            let now = std::time::Instant::now();
            ctx.last_announce = now;
            ctx.last_gossip = now;
            ctx.last_redial = now;
            if ctx
                .directory
                .publish_room(room, &ctx.room_params, &ctx.peer_id, &ctx.last_addrs)
                .is_err()
            {
                warn!(target: "clicker", "discovery: room publish failed on switch");
            }
            if ctx.roster.announce().is_err() {
                warn!(target: "clicker", "discovery: announce failed on switch");
            }
            ctx.cmd_tx
                .send(p2p::Command::FetchRoster {
                    lobby: room.to_string(),
                })
                .ok();
            ctx.cmd_tx
                .send(p2p::Command::FetchHistory {
                    lobby: room.to_string(),
                    chunk: constants::SNAPSHOT_CHUNK,
                })
                .ok();
            info!(target: "clicker", room = %room, "discovery: switched room");
            true
        }
        Err(e) => {
            warn!(target: "clicker", error = %e, "discovery: roster switch failed");
            false
        }
    }
}

// needed helper: drives the connected roster steady-state loop
async fn drive_roster(ctx: &mut RunContext) {
    let mut roster_topic = format!("clicker/{}/roster", ctx.room);
    loop {
        while let Ok(room) = ctx.room_requests.try_recv() {
            if discovery::should_switch(&ctx.room, &room) {
                ctx.pending_switch = Some((room, 0));
            }
        }
        if let Some((room, attempts)) = ctx.pending_switch.take() {
            if attempts == 0 || attempts % 10 == 0 {
                info!(target: "clicker", room = %room, attempts, "discovery: switching room");
            }
            if switch_room(ctx, &room).await {
                roster_topic = format!("clicker/{}/roster", ctx.room);
            } else {
                ctx.pending_switch = Some((room, attempts.saturating_add(1)));
            }
        }
        while let Ok((peer, up)) = ctx.links.try_recv() {
            if up {
                let fresh = !ctx.connected.contains_key(&peer);
                count_link(ctx, peer.clone());
                if fresh {
                    send_pex_ask(&ctx.cmd_tx, &peer);
                    ctx.pex_asked.insert(peer, std::time::Instant::now());
                }
            } else {
                decrement_link(ctx, &peer);
            }
        }
        let mut hub = DialHub {
            cmd_tx: &ctx.cmd_tx,
            attempted: &mut ctx.attempted,
            connected: &ctx.connected,
            staggers: &mut ctx.staggers,
            own_peer_id: &ctx.peer_id,
            mode: ctx.transport,
        };
        drain_lobby_events(
            &mut hub,
            &mut ctx.lobby_events,
            &mut ctx.pex,
            &mut ctx.pex_inbox,
        );
        drain_pex(ctx);
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
        if ctx.last_pex.elapsed().as_secs() >= discovery::PEX_INTERVAL_SECS {
            ctx.last_pex = std::time::Instant::now();
            reask_pex(ctx);
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
            ctx.directory.slots = slots;
            send_merged_directory(ctx);
            for hint in directory_hints(&ctx.directory.slots) {
                ctx.pex.insert(hint.clone());
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

// needed helper: redials known-but-disconnected roster peers and hint peers
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
    let hints: Vec<discovery::PeerHint> = ctx.pex.values().cloned().collect();
    for hint in &hints {
        if ctx.connected.contains_key(&hint.peer_id) {
            continue;
        }
        dial_hint(
            &ctx.cmd_tx,
            &mut ctx.attempted,
            &ctx.connected,
            &mut ctx.staggers,
            &ctx.peer_id,
            ctx.transport,
            hint,
        );
    }
}

// needed helper: refreshes roster addrs when the observed address changes
fn refresh_observed(ctx: &mut RunContext) {
    let seen = ctx.observed.borrow_and_update().clone().unwrap_or_default();
    let filtered = discovery::observed_addrs(seen.clone(), &ctx.last_addrs);
    if filtered != seen {
        info!(target: "clicker", seen = ?seen, kept = ?filtered, "discovery: ignored non-listen observed addr");
    }
    let dialable_seen = dialable(filtered);
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
            return Some(resolve_requested(
                directory,
                &config.namespace,
                config.params_override.as_deref(),
                room,
                peer_id,
                addrs,
            ));
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
                if discovery::auto_join(std::env::var("CLICKER_NO_AUTOJOIN").ok().map(|_| true))
                    && let Some((room, entry)) = discovery::pick_room(&slots, config.since_secs)
                {
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
        let request = tokio::select! {
            biased;
            request = config.room_requests.recv() => request,
            () = tokio::time::sleep(Duration::from_secs(DIRECTORY_TICK_SECS)) => None,
        };
        if let Some(room) = request {
            return Some(resolve_requested(
                directory,
                &config.namespace,
                config.params_override.as_deref(),
                room,
                peer_id,
                addrs,
            ));
        }
    }
}

// needed helper: publishes a user-requested room and resolves its join params
fn resolve_requested(
    directory: &discovery::Directory,
    namespace: &str,
    params_override: Option<&str>,
    room: String,
    peer_id: &str,
    addrs: &[String],
) -> (String, Vec<u8>) {
    let params = discovery::resolve_params(namespace, &room, params_override);
    if directory
        .publish_room(&room, &params, peer_id, addrs)
        .is_err()
    {
        warn!(target: "clicker", "discovery: room publish failed");
    }
    (room, params)
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
        .iter()
        .map(|(room, entry)| discovery::PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: vec![room.clone()],
            updated_at: entry.updated_at,
        })
        .collect();
    discovery::merge_peer_hints(hints)
}

// needed helper: shared dial arguments for lobby event drains
struct DialHub<'a> {
    cmd_tx: &'a tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    attempted: &'a mut std::collections::HashMap<String, std::time::Instant>,
    connected: &'a ConnectedMap,
    staggers: &'a mut StaggerMap,
    own_peer_id: &'a str,
    mode: p2p::TransportMode,
}

// needed helper: drains libp2p lobby/provider/gossip events into dials
fn drain_lobby_events(
    hub: &mut DialHub,
    lobby_events: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event<clicker::CursorMsg>>,
    pex: &mut discovery::HintStore,
    pex_inbox: &mut Vec<(String, clicker::CursorMsg)>,
) {
    while let Ok(event) = lobby_events.try_recv() {
        match event {
            p2p::Event::Message { from, payload }
                if matches!(
                    payload,
                    clicker::CursorMsg::PexAsk { .. } | clicker::CursorMsg::PexResp { .. }
                ) =>
            {
                pex_inbox.push((from, payload));
            }
            p2p::Event::LobbyProviders { peers, .. } => {
                for peer in peers {
                    dial_hint(
                        hub.cmd_tx,
                        &mut *hub.attempted,
                        hub.connected,
                        &mut *hub.staggers,
                        hub.own_peer_id,
                        hub.mode,
                        &discovery::PeerHint {
                            peer_id: peer,
                            addrs: Vec::new(),
                            rooms: Vec::new(),
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
                    let hint = discovery::PeerHint {
                        peer_id: entry.peer_id.clone(),
                        addrs: entry.addrs.clone(),
                        rooms: Vec::new(),
                        updated_at: entry.updated_at,
                    };
                    pex.insert(hint.clone());
                    dial_hint(
                        hub.cmd_tx,
                        &mut *hub.attempted,
                        hub.connected,
                        &mut *hub.staggers,
                        hub.own_peer_id,
                        hub.mode,
                        &hint,
                    );
                }
            }
            p2p::Event::PeerConnected(peer) => {
                hub.attempted.insert(peer, std::time::Instant::now());
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

// needed helper: publishes the fresh foreign peer ids so the join gate knows the full set
fn send_expected(
    expected_tx: &tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    slots: &discovery::RosterState,
    own: discovery::PlayerId,
) {
    let now = epoch_secs();
    let mut peers: Vec<String> = slots
        .iter()
        .filter(|(id, entry)| {
            **id != own
                && !entry.peer_id.is_empty()
                && now.saturating_sub(entry.updated_at) <= constants::STALE_ENTRY_SECS
        })
        .map(|(_, entry)| entry.peer_id.clone())
        .collect();
    peers.sort();
    peers.dedup();
    info!(target: "clicker", peers = peers.len(), "discovery: expected set published");
    expected_tx.send(peers).ok();
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
