use std::collections::{BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::p2p;

use super::auto_join::auto_join;
use super::bootstrap_node::bootstrap_node;
use super::constants;
use super::decide_dial::decide_dial;
use super::dial_decision::DialDecision;
use super::dir_params::dir_params;
use super::directory::Directory;
use super::directory_entry::DirectoryEntry;
use super::directory_state::DirectoryState;
use super::hint_store::HintStore;
use super::hint_union::hint_union;
use super::merge_peer_hints::merge_peer_hints;
use super::node_mode::NodeMode;
use super::observed_addrs::observed_addrs;
use super::peer_entry::PeerEntry;
use super::peer_hint::PeerHint;
use super::pick_room::pick_room;
use super::player_id::PlayerId;
use super::rank_addrs::rank_addrs;
use super::resolve_params::resolve_params;
use super::roster::Roster;
use super::roster_state::RosterState;
use super::run_config::RunConfig;
use super::should_switch::should_switch;
use super::stagger_due::stagger_due;

type ConnectedMap = HashMap<String, u32>;
type StaggerMap = HashMap<String, (VecDeque<String>, Instant)>;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PexMsg {
    Ask,
    Resp {
        peers: Vec<PeerHint>,
        rooms: Vec<(String, DirectoryEntry)>,
    },
}

struct RunContext {
    net_tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    roster: Roster,
    directory: Directory,
    room: String,
    room_params: Vec<u8>,
    peer_id: String,
    tap_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    observed_rx: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    namespace: String,
    params_override: Option<String>,
    last_addrs: Vec<String>,
    last_announce: Instant,
    last_gossip: Instant,
    last_redial: Instant,
    last_directory: Instant,
    last_pex: Instant,
    connected: ConnectedMap,
    attempted: HashMap<String, Instant>,
    staggers: StaggerMap,
    transport: p2p::TransportMode,
    directory_tx: tokio::sync::mpsc::UnboundedSender<DirectoryState>,
    expected_tx: tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    pex: HintStore,
    known_rooms: BTreeMap<String, DirectoryEntry>,
    room_requests: tokio::sync::mpsc::UnboundedReceiver<String>,
    room_tx: tokio::sync::watch::Sender<Option<String>>,
    ws_port: u16,
    own: PlayerId,
    pending_switch: Option<(String, u32)>,
    _node_guard: Option<tempfile::TempDir>,
}

pub async fn run(mut config: RunConfig) {
    let Some((peer_id, addrs)) = wait_ready(&mut config.ready_rx).await else {
        warn!(target: "room_lobby", "discovery: no libp2p ready signal, discovery disabled");
        return;
    };
    let addrs = dialable(addrs);
    let (node_guard, ws_port) = match start_node(config.node).await {
        Ok(node) => node,
        Err(e) => {
            warn!(target: "room_lobby", error = %e, "discovery: node bootstrap failed");
            return;
        }
    };
    let mut directory = connect_directory_retry(ws_port, &config.namespace).await;
    info!(target: "room_lobby", key = %directory.contract_key, "discovery: directory connected");
    let Some((room, room_params)) =
        resolve_room(&mut config, &mut directory, &peer_id, &addrs).await
    else {
        warn!(target: "room_lobby", "discovery: no room resolved, discovery disabled");
        return;
    };
    info!(target: "room_lobby", room = %room, "discovery: room resolved");
    config.room_tx.send_replace(Some(room.clone()));
    publish_pre_get_union(&config.expected_tx, &directory.slots, &peer_id, &room);
    let roster =
        connect_roster_retry(ws_port, &room, &room_params, config.own, &peer_id, &addrs).await;
    info!(target: "room_lobby", key = %roster.contract_key, own = *config.own, "discovery: roster connected");
    send_expected(&config.expected_tx, &roster.slots, config.own);
    subscribe_topics(&config.net_tx, &config.namespace, &room);

    let mut attempted: HashMap<String, Instant> = HashMap::new();
    let mut staggers: StaggerMap = HashMap::new();
    let connected: ConnectedMap = HashMap::new();
    dial_known(
        &config.net_tx,
        &connected,
        &mut staggers,
        &roster.slots,
        config.own,
        config.transport,
    );
    dial_directory_publishers(
        &config.net_tx,
        &mut attempted,
        &connected,
        &mut staggers,
        &peer_id,
        config.transport,
        &directory.slots,
    );
    if roster.announce().is_err() {
        warn!(target: "room_lobby", "discovery: initial announce failed");
    }
    let now = Instant::now();
    let mut ctx = RunContext {
        net_tx: config.net_tx,
        roster,
        directory,
        room,
        room_params,
        peer_id,
        tap_rx: config.tap_rx,
        observed_rx: config.observed_rx,
        namespace: config.namespace,
        params_override: config.params_override,
        last_addrs: addrs,
        last_announce: now,
        last_gossip: now,
        last_redial: now,
        last_directory: now,
        last_pex: now,
        connected,
        attempted,
        staggers,
        transport: config.transport,
        directory_tx: config.directory_tx,
        expected_tx: config.expected_tx,
        pex: HintStore::default(),
        known_rooms: BTreeMap::new(),
        room_requests: config.room_requests,
        room_tx: config.room_tx,
        ws_port,
        own: config.own,
        pending_switch: None,
        _node_guard: node_guard,
    };
    drive_roster(&mut ctx).await;
}

// needed helper: starts the embedded node or reuses an external websocket port
async fn start_node(
    mode: NodeMode,
) -> Result<(Option<tempfile::TempDir>, u16), super::error::Error> {
    match mode {
        NodeMode::Embedded => {
            let (guard, port) = bootstrap_node().await?;
            Ok((Some(guard), port))
        }
        NodeMode::External { ws_port } => Ok((None, ws_port)),
    }
}

// needed helper: subscribes the libp2p pex + roster topics for a room
fn subscribe_topics(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    namespace: &str,
    room: &str,
) {
    for topic in [pex_topic(namespace), roster_topic(namespace, room)] {
        net_tx.send(p2p::NetCommand::Subscribe { topic }).ok();
    }
}

// needed helper: libp2p gossip topic for peer exchange
fn pex_topic(namespace: &str) -> String {
    format!("{namespace}/{}", constants::PEX_TOPIC_SUFFIX)
}

// needed helper: libp2p gossip topic for roster mirroring
fn roster_topic(namespace: &str, room: &str) -> String {
    format!("{namespace}/{room}/{}", constants::ROSTER_TOPIC_SUFFIX)
}

// needed helper: publishes the pre-Get hint union so the gate never waits on a stalled roster Get
fn publish_pre_get_union(
    expected_tx: &tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    slots: &DirectoryState,
    peer_id: &str,
    room: &str,
) {
    let union = hint_union(slots, &HintStore::default(), peer_id, epoch_secs());
    info!(target: "room_lobby", room = %room, peers = union.len(), "discovery: expected hint-union published before roster get");
    expected_tx.send(union).ok();
}

// needed helper: connects the roster contract, retrying until the node answers
async fn connect_roster_retry(
    ws_port: u16,
    room: &str,
    room_params: &[u8],
    own: PlayerId,
    peer_id: &str,
    addrs: &[String],
) -> Roster {
    loop {
        let attempt = Instant::now();
        match Roster::connect(
            "127.0.0.1",
            ws_port,
            super::contract_wasm::contract_wasm(),
            room_params,
            own,
            peer_id,
            addrs,
        )
        .await
        {
            Ok(roster) => {
                info!(target: "room_lobby", room = %room, slots = roster.slots.len(), elapsed_ms = attempt.elapsed().as_millis(), "discovery: roster fetched");
                return roster;
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: roster connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// needed helper: connects the directory contract, retrying until the node answers
async fn connect_directory_retry(ws_port: u16, namespace: &str) -> Directory {
    loop {
        match Directory::connect(
            "127.0.0.1",
            ws_port,
            super::contract_wasm::contract_wasm(),
            &dir_params(namespace),
        )
        .await
        {
            Ok(directory) => return directory,
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: directory connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// needed helper: publishes a PEX ask or response on the gossip topic
fn publish_pex(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    namespace: &str,
    msg: &PexMsg,
) {
    let data = bincode::serialize(msg).unwrap_or_default();
    net_tx
        .send(p2p::NetCommand::Publish {
            topic: pex_topic(namespace),
            data,
        })
        .ok();
}

// needed helper: drains connection and dial-failure events into link counts and prune sets
fn drain_link_events(ctx: &mut RunContext) {
    while let Ok(event) = ctx.tap_rx.try_recv() {
        match event {
            p2p::TapEvent::PeerConnected(peer) => {
                let fresh = !ctx.connected.contains_key(&peer);
                count_link(ctx, peer.clone());
                if fresh {
                    ctx.attempted.insert(peer, Instant::now());
                }
            }
            p2p::TapEvent::PeerDisconnected(peer) => {
                decrement_link(ctx, &peer);
            }
            p2p::TapEvent::DialFailed { peer_id, .. } => {
                ctx.staggers.remove(&peer_id);
                ctx.pex.remove(peer_id.as_str());
                warn!(target: "room_lobby", peer = %peer_id, "discovery: pruned dial-failed peer");
            }
            other => drain_tap(ctx, other),
        }
    }
}

// needed helper: handles one gossip/provider tap event
fn drain_tap(ctx: &mut RunContext, event: p2p::TapEvent) {
    match event {
        p2p::TapEvent::Gossip { topic, from, data } => {
            if topic == pex_topic(&ctx.namespace) {
                absorb_pex(ctx, &from, &data);
            } else if is_roster_topic(&ctx.namespace, &topic) {
                absorb_roster_gossip(ctx, &from, &data);
            }
        }
        p2p::TapEvent::LobbyProviders { peers, .. } => {
            for peer in peers {
                dial_hint(
                    ctx,
                    &PeerHint {
                        peer_id: peer,
                        addrs: Vec::new(),
                        rooms: Vec::new(),
                        updated_at: epoch_secs(),
                    },
                );
            }
        }
        _ => {}
    }
}

// needed helper: checks whether a gossip topic is a roster mirror topic
fn is_roster_topic(namespace: &str, topic: &str) -> bool {
    topic.starts_with(&format!("{namespace}/"))
        && topic.ends_with(&format!("/{}", constants::ROSTER_TOPIC_SUFFIX))
}

// needed helper: absorbs a roster gossip mirror into hints and dials new peers
fn absorb_roster_gossip(ctx: &mut RunContext, from: &str, data: &[u8]) {
    let slots: RosterState = bincode::deserialize(data).unwrap_or_default();
    for entry in slots.values() {
        if entry.peer_id == from || entry.peer_id.is_empty() {
            continue;
        }
        let hint = PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: Vec::new(),
            updated_at: entry.updated_at,
        };
        ctx.pex.insert(hint.clone());
        dial_hint(ctx, &hint);
    }
}

// needed helper: answers a PEX ask and absorbs a PEX response
fn absorb_pex(ctx: &mut RunContext, from: &str, data: &[u8]) {
    let Ok(msg) = bincode::deserialize::<PexMsg>(data) else {
        return;
    };
    match msg {
        PexMsg::Ask => publish_pex(&ctx.net_tx, &ctx.namespace, &pex_response(ctx)),
        PexMsg::Resp { peers, rooms } => absorb_pex_resp(ctx, from, &peers, &rooms),
    }
}

// needed helper: builds our PEX response from roster slots and known rooms
fn pex_response(ctx: &RunContext) -> PexMsg {
    let mut peers: Vec<PeerHint> = ctx
        .roster
        .slots
        .values()
        .map(|entry| PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: Vec::new(),
            updated_at: entry.updated_at,
        })
        .collect();
    for hint in ctx.pex.values() {
        peers.push(hint.clone());
    }
    let peers = merge_peer_hints(peers)
        .into_iter()
        .take(constants::PEX_MAX_HINTS)
        .collect();
    let rooms: Vec<(String, DirectoryEntry)> = ctx
        .known_rooms
        .iter()
        .map(|(name, entry)| (name.clone(), entry.clone()))
        .chain([(
            ctx.room.clone(),
            DirectoryEntry {
                params: ctx.room_params.clone(),
                peer_id: ctx.peer_id.clone(),
                addrs: ctx.last_addrs.clone(),
                updated_at: epoch_secs(),
            },
        )])
        .take(constants::PEX_MAX_ROOMS)
        .collect();
    PexMsg::Resp { peers, rooms }
}

// needed helper: merges PEX answers into the hint store, dials the unknown
fn absorb_pex_resp(
    ctx: &mut RunContext,
    _from: &str,
    peers: &[PeerHint],
    rooms: &[(String, DirectoryEntry)],
) {
    for hint in peers.iter().take(constants::PEX_MAX_HINTS) {
        if hint.peer_id == ctx.peer_id {
            continue;
        }
        ctx.pex.insert(hint.clone());
        dial_hint(ctx, hint);
    }
    for (name, entry) in rooms.iter().take(constants::PEX_MAX_ROOMS) {
        if name.is_empty() || entry.params.is_empty() {
            continue;
        }
        let keep = ctx
            .known_rooms
            .get(name)
            .is_none_or(|known| entry.updated_at >= known.updated_at);
        if keep {
            if !ctx.known_rooms.contains_key(name)
                && ctx.known_rooms.len() >= constants::PEX_MAX_ROOMS
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

// needed helper: counts one open connection per peer so sub-connection churn never reads as a full drop
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
    send_hint_union(ctx);
    let params = resolve_params(&ctx.namespace, room, ctx.params_override.as_deref());
    let attempt = Instant::now();
    match Roster::connect(
        "127.0.0.1",
        ctx.ws_port,
        super::contract_wasm::contract_wasm(),
        &params,
        ctx.own,
        &ctx.peer_id,
        &ctx.last_addrs,
    )
    .await
    {
        Ok(roster) => {
            info!(target: "room_lobby", room = %room, slots = roster.slots.len(), elapsed_ms = attempt.elapsed().as_millis(), "discovery: roster fetched");
            ctx.roster = roster;
            send_expected(&ctx.expected_tx, &ctx.roster.slots, ctx.own);
            ctx.room = room.to_string();
            ctx.room_params = params;
            ctx.room_tx.send_replace(Some(room.to_string()));
            ctx.attempted.clear();
            ctx.staggers.clear();
            let now = Instant::now();
            ctx.last_announce = now;
            ctx.last_gossip = now;
            ctx.last_redial = now;
            subscribe_topics(&ctx.net_tx, &ctx.namespace, room);
            if ctx
                .directory
                .publish_room(room, &ctx.room_params, &ctx.peer_id, &ctx.last_addrs)
                .is_err()
            {
                warn!(target: "room_lobby", "discovery: room publish failed on switch");
            }
            if ctx.roster.announce().is_err() {
                warn!(target: "room_lobby", "discovery: announce failed on switch");
            }
            info!(target: "room_lobby", room = %room, "discovery: switched room");
            true
        }
        Err(e) => {
            warn!(target: "room_lobby", error = %e, "discovery: roster switch failed");
            false
        }
    }
}

// needed helper: drives the connected roster steady-state loop
async fn drive_roster(ctx: &mut RunContext) {
    loop {
        while let Ok(room) = ctx.room_requests.try_recv() {
            if should_switch(&ctx.room, &room) {
                ctx.pending_switch = Some((room, 0));
            }
        }
        if let Some((room, attempts)) = ctx.pending_switch.take() {
            if attempts == 0 || attempts % 10 == 0 {
                info!(target: "room_lobby", room = %room, attempts, "discovery: switching room");
            }
            if !switch_room(ctx, &room).await {
                ctx.pending_switch = Some((room, attempts.saturating_add(1)));
            }
        }
        drain_link_events(ctx);
        match ctx.roster.poll().await {
            Ok(fresh) => {
                for entry in fresh {
                    dial_entry(ctx, &entry);
                }
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: poll failed");
            }
        }
        if ctx.last_directory.elapsed().as_secs() >= constants::DIRECTORY_TICK_SECS {
            ctx.last_directory = Instant::now();
            refresh_directory(ctx).await;
        }
        if ctx.last_gossip.elapsed().as_secs_f64() >= constants::ROSTER_HEARTBEAT_SECS {
            ctx.last_gossip = Instant::now();
            publish_slots(ctx);
        }
        if ctx.last_redial.elapsed().as_secs() >= constants::REDIAL_SECS {
            ctx.last_redial = Instant::now();
            redial_missing(ctx);
        }
        if ctx.last_pex.elapsed().as_secs() >= constants::PEX_INTERVAL_SECS {
            ctx.last_pex = Instant::now();
            ctx.pex.prune(epoch_secs());
            publish_pex(&ctx.net_tx, &ctx.namespace, &PexMsg::Ask);
        }
        fire_due_staggers(ctx);
        if ctx.observed_rx.has_changed().unwrap_or(false) {
            refresh_observed(ctx);
        }
        if ctx.roster.bridge_tick(Instant::now()).is_err() {
            warn!(target: "room_lobby", "discovery: bridge failed");
        }
        if ctx.roster.announce().is_err() {
            warn!(target: "room_lobby", "discovery: announce failed");
        }
        tokio::time::sleep(Duration::from_secs(constants::TICK_SECS)).await;
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
                dial_hint(ctx, &hint);
            }
        }
        Err(e) => {
            warn!(target: "room_lobby", error = %e, "discovery: directory poll failed");
        }
    }
    if ctx
        .directory
        .publish_room(&ctx.room, &ctx.room_params, &ctx.peer_id, &ctx.last_addrs)
        .is_err()
    {
        warn!(target: "room_lobby", "discovery: directory refresh failed");
    }
    if ctx.directory.bridge_tick(Instant::now()).is_err() {
        warn!(target: "room_lobby", "discovery: directory bridge failed");
    }
}

// needed helper: redials known-but-disconnected roster peers and hint peers
fn redial_missing(ctx: &mut RunContext) {
    let entries: Vec<PeerEntry> = ctx
        .roster
        .slots
        .iter()
        .filter(|(id, _)| **id != ctx.own)
        .map(|(_, entry)| entry.clone())
        .collect();
    for entry in &entries {
        if ctx.connected.contains_key(&entry.peer_id) {
            continue;
        }
        dial_entry(ctx, entry);
    }
    let hints: Vec<PeerHint> = ctx.pex.values().cloned().collect();
    for hint in &hints {
        if ctx.connected.contains_key(&hint.peer_id) {
            continue;
        }
        dial_hint(ctx, hint);
    }
}

// needed helper: refreshes roster addrs when the observed address changes
fn refresh_observed(ctx: &mut RunContext) {
    let seen = ctx
        .observed_rx
        .borrow_and_update()
        .clone()
        .unwrap_or_default();
    let filtered = observed_addrs(seen.clone(), &ctx.last_addrs);
    if filtered != seen {
        info!(target: "room_lobby", seen = ?seen, kept = ?filtered, "discovery: ignored non-listen observed addr");
    }
    let dialable_seen = dialable(filtered);
    if dialable_seen.is_empty() || dialable_seen == ctx.last_addrs {
        return;
    }
    ctx.last_addrs.clone_from(&dialable_seen);
    ctx.roster.addrs = dialable_seen;
    ctx.last_announce = Instant::now();
    if ctx.roster.announce().is_err() {
        warn!(target: "room_lobby", "discovery: re-announce failed");
    }
}

// needed helper: resolves the room via forced lobby, directory list, or auto-join
async fn resolve_room(
    config: &mut RunConfig,
    directory: &mut Directory,
    peer_id: &str,
    addrs: &[String],
) -> Option<(String, Vec<u8>)> {
    if let Some(room) = config.lobby.as_deref() {
        let params = resolve_params(&config.namespace, room, config.params_override.as_deref());
        if directory
            .publish_room(room, &params, peer_id, addrs)
            .is_err()
        {
            warn!(target: "room_lobby", "discovery: room publish failed");
        }
        return Some((room.to_string(), params));
    }
    let deadline = Instant::now().checked_add(Duration::from_secs(constants::DISCOVERY_SECS))?;
    let mut attempted: HashMap<String, Instant> = HashMap::new();
    let connected: ConnectedMap = HashMap::new();
    let mut staggers: StaggerMap = HashMap::new();
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
        match directory.poll().await {
            Ok(slots) => {
                config.directory_tx.send(slots.clone()).ok();
                for hint in directory_hints(&slots) {
                    dial_hint_raw(
                        &config.net_tx,
                        &mut attempted,
                        &connected,
                        &mut staggers,
                        peer_id,
                        config.transport,
                        &hint,
                    );
                }
                if auto_join(std::env::var("ROOM_LOBBY_NO_AUTOJOIN").ok().map(|_| true))
                    && let Some((room, entry)) = pick_room(&slots, config.since_secs)
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
                warn!(target: "room_lobby", error = %e, "discovery: room poll failed");
            }
        }
        if Instant::now() >= deadline {
            return None;
        }
        fire_due_staggers_raw(&config.net_tx, &connected, &mut staggers);
        let request = tokio::select! {
            biased;
            request = config.room_requests.recv() => request,
            () = tokio::time::sleep(Duration::from_secs(constants::DIRECTORY_TICK_SECS)) => None,
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
    directory: &Directory,
    namespace: &str,
    params_override: Option<&str>,
    room: String,
    peer_id: &str,
    addrs: &[String],
) -> (String, Vec<u8>) {
    let params = resolve_params(namespace, &room, params_override);
    if directory
        .publish_room(&room, &params, peer_id, addrs)
        .is_err()
    {
        warn!(target: "room_lobby", "discovery: room publish failed");
    }
    (room, params)
}

// needed helper: picks join params from the directory entry or recomputes them
fn room_params(
    namespace: &str,
    room: &str,
    params_override: Option<&str>,
    entry: DirectoryEntry,
) -> Vec<u8> {
    if entry.params.is_empty() {
        resolve_params(namespace, room, params_override)
    } else {
        entry.params
    }
}

// needed helper: converts directory entries into dialable peer hints
fn directory_hints(slots: &DirectoryState) -> Vec<PeerHint> {
    let hints: Vec<PeerHint> = slots
        .iter()
        .map(|(room, entry)| PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: vec![room.clone()],
            updated_at: entry.updated_at,
        })
        .collect();
    merge_peer_hints(hints)
}

// needed helper: dials one roster entry with the deterministic tie-break
fn dial_entry(ctx: &mut RunContext, entry: &PeerEntry) {
    if entry.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let age = ctx.attempted.get(&entry.peer_id).and_then(|seen| {
        Instant::now()
            .checked_duration_since(*seen)
            .map(|d| d.as_secs())
    });
    if age.is_some_and(|seen_secs| seen_secs < constants::REDIAL_SECS) {
        return;
    }
    let decision = decide_dial(&ctx.peer_id, &entry.peer_id, age);
    apply_decision(ctx, entry, decision);
}

// needed helper: applies a dial decision for one peer entry
fn apply_decision(ctx: &mut RunContext, entry: &PeerEntry, decision: DialDecision) {
    match decision {
        DialDecision::Wait => {
            ctx.attempted
                .entry(entry.peer_id.clone())
                .or_insert_with(Instant::now);
        }
        DialDecision::Dial => {
            ctx.attempted.insert(entry.peer_id.clone(), Instant::now());
            dial_preferred(ctx, entry);
        }
        DialDecision::ForceDial => {
            ctx.attempted.insert(entry.peer_id.clone(), Instant::now());
            force_entry(ctx, entry);
        }
    }
}

// needed helper: dials one bootstrapped peer hint discovered via directory or libp2p
fn dial_hint(ctx: &mut RunContext, hint: &PeerHint) {
    if hint.peer_id.is_empty() || hint.peer_id == ctx.peer_id || hint.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(hint.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let entry = PeerEntry {
        peer_id: hint.peer_id.clone(),
        addrs: hint.addrs.clone(),
        updated_at: hint.updated_at,
    };
    dial_entry(ctx, &entry);
}

// needed helper: dials the best-ranked addr now and staggers the rest
fn dial_preferred(ctx: &mut RunContext, entry: &PeerEntry) {
    if ctx.connected.contains_key(&entry.peer_id) {
        ctx.staggers.remove(&entry.peer_id);
        return;
    }
    let ranked = rank_addrs(&with_loopback(&entry.addrs), ctx.transport);
    let mut queue: VecDeque<String> = ranked.into();
    let Some(first) = queue.pop_front() else {
        return;
    };
    dial_addrs(&ctx.net_tx, &entry.peer_id, &[first], &entry.addrs);
    if queue.is_empty() {
        ctx.staggers.remove(&entry.peer_id);
        return;
    }
    let due = Instant::now()
        .checked_add(Duration::from_secs(constants::STAGGER_SECS))
        .unwrap_or_else(Instant::now);
    ctx.staggers.insert(entry.peer_id.clone(), (queue, due));
}

// needed helper: fires due staggered dials for peers still unreached
fn fire_due_staggers(ctx: &mut RunContext) {
    let due = stagger_due(Instant::now(), &ctx.connected, &mut ctx.staggers);
    for (peer, addr) in due {
        info!(target: "room_lobby", peer = %peer, addr = %addr, "discovery: staggered dialing peer");
        ctx.net_tx
            .send(p2p::NetCommand::DialForce {
                peer_id: peer,
                addrs: vec![addr],
            })
            .ok();
    }
}

// needed helper: sends one dial plus kad seeding for an addr set
fn dial_addrs(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    peer_id: &str,
    addrs: &[String],
    kad_addrs: &[String],
) {
    if addrs.is_empty() {
        return;
    }
    info!(target: "room_lobby", peer = %peer_id, addrs = ?addrs, "discovery: dialing peer");
    net_tx
        .send(p2p::NetCommand::Dial {
            peer_id: peer_id.to_string(),
            addrs: addrs.to_vec(),
        })
        .ok();
    net_tx
        .send(p2p::NetCommand::AddKadPeer {
            peer_id: peer_id.to_string(),
            addrs: kad_addrs.to_vec(),
        })
        .ok();
}

// needed helper: mirrors known roster slots onto the gossip roster topic
fn publish_slots(ctx: &RunContext) {
    let data = bincode::serialize(&ctx.roster.slots).unwrap_or_default();
    ctx.net_tx
        .send(p2p::NetCommand::Publish {
            topic: roster_topic(&ctx.namespace, &ctx.room),
            data,
        })
        .ok();
}

// needed helper: force-dials one peer entry past stuck swarm dial state
fn force_entry(ctx: &RunContext, entry: &PeerEntry) {
    if entry.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    info!(target: "room_lobby", peer = %entry.peer_id, "discovery: force-dialing peer");
    ctx.net_tx
        .send(p2p::NetCommand::DialForce {
            peer_id: entry.peer_id.clone(),
            addrs: with_loopback(&entry.addrs),
        })
        .ok();
}

// needed helper: publishes the pre-Get hint union and dials it
fn send_hint_union(ctx: &mut RunContext) {
    let union = hint_union(&ctx.directory.slots, &ctx.pex, &ctx.peer_id, epoch_secs());
    info!(target: "room_lobby", peers = union.len(), "discovery: expected hint-union published");
    ctx.expected_tx.send(union).ok();
    let mut hints = directory_hints(&ctx.directory.slots);
    hints.extend(ctx.pex.values().cloned());
    for hint in merge_peer_hints(hints) {
        dial_hint(ctx, &hint);
    }
}

// needed helper: publishes the fresh foreign peer ids so the join gate knows the full set
fn send_expected(
    expected_tx: &tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    slots: &RosterState,
    own: PlayerId,
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
    info!(target: "room_lobby", peers = peers.len(), "discovery: expected set published");
    expected_tx.send(peers).ok();
}

// needed helper: dials directory publishers at connect time so joins never wait on the roster Get
fn dial_directory_publishers(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    attempted: &mut HashMap<String, Instant>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    peer_id: &str,
    mode: p2p::TransportMode,
    slots: &DirectoryState,
) {
    for hint in directory_hints(slots) {
        dial_hint_raw(net_tx, attempted, connected, staggers, peer_id, mode, &hint);
    }
}

// needed helper: dials foreign roster entries already present at connect time
fn dial_known(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    slots: &RosterState,
    own: PlayerId,
    mode: p2p::TransportMode,
) {
    for (id, entry) in slots {
        if *id == own {
            continue;
        }
        dial_preferred_raw(net_tx, connected, staggers, mode, entry);
    }
}

// needed helper: dials one bootstrapped hint without a RunContext
fn dial_hint_raw(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    attempted: &mut HashMap<String, Instant>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    peer_id: &str,
    mode: p2p::TransportMode,
    hint: &PeerHint,
) {
    if hint.peer_id.is_empty() || hint.peer_id == peer_id || hint.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(hint.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let entry = PeerEntry {
        peer_id: hint.peer_id.clone(),
        addrs: hint.addrs.clone(),
        updated_at: hint.updated_at,
    };
    let age = attempted.get(&entry.peer_id).and_then(|seen| {
        Instant::now()
            .checked_duration_since(*seen)
            .map(|d| d.as_secs())
    });
    if age.is_some_and(|seen_secs| seen_secs < constants::REDIAL_SECS) {
        return;
    }
    match decide_dial(peer_id, &entry.peer_id, age) {
        DialDecision::Wait => {
            attempted
                .entry(entry.peer_id.clone())
                .or_insert_with(Instant::now);
        }
        DialDecision::Dial => {
            attempted.insert(entry.peer_id.clone(), Instant::now());
            dial_preferred_raw(net_tx, connected, staggers, mode, &entry);
        }
        DialDecision::ForceDial => {
            attempted.insert(entry.peer_id.clone(), Instant::now());
            net_tx
                .send(p2p::NetCommand::DialForce {
                    peer_id: entry.peer_id.clone(),
                    addrs: with_loopback(&entry.addrs),
                })
                .ok();
        }
    }
}

// needed helper: dials the best-ranked addr now and staggers the rest (no context)
fn dial_preferred_raw(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    mode: p2p::TransportMode,
    entry: &PeerEntry,
) {
    if connected.contains_key(&entry.peer_id) {
        staggers.remove(&entry.peer_id);
        return;
    }
    let ranked = rank_addrs(&with_loopback(&entry.addrs), mode);
    let mut queue: VecDeque<String> = ranked.into();
    let Some(first) = queue.pop_front() else {
        return;
    };
    dial_addrs(net_tx, &entry.peer_id, &[first], &entry.addrs);
    if queue.is_empty() {
        staggers.remove(&entry.peer_id);
        return;
    }
    let due = Instant::now()
        .checked_add(Duration::from_secs(constants::STAGGER_SECS))
        .unwrap_or_else(Instant::now);
    staggers.insert(entry.peer_id.clone(), (queue, due));
}

// needed helper: fires due staggered dials for peers still unreached (no context)
fn fire_due_staggers_raw(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
) {
    for (peer, addr) in stagger_due(Instant::now(), connected, staggers) {
        net_tx
            .send(p2p::NetCommand::DialForce {
                peer_id: peer,
                addrs: vec![addr],
            })
            .ok();
    }
}

// needed helper: waits for the libp2p ready signal carrying our listen addrs
async fn wait_ready(
    ready: &mut tokio::sync::watch::Receiver<Option<p2p::Ready>>,
) -> Option<(String, Vec<String>)> {
    let deadline = tokio::time::Instant::now()
        .checked_add(Duration::from_secs(constants::READY_TIMEOUT_SECS))?;
    loop {
        let value = ready.borrow().clone();
        if let Some(info) = value {
            return Some((info.peer_id, info.addrs));
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return None;
        }
        if tokio::time::timeout(remaining, ready.changed())
            .await
            .is_err()
        {
            return ready.borrow().clone().map(|r| (r.peer_id, r.addrs));
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
