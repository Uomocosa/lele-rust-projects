use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use tracing::{info, warn};

use super::super::gossip::hint_store::HintStore;
use super::super::params::remote_peer_id::RemotePeerId;
use super::connect_catalog_retry::connect_catalog_retry;
use super::connect_roster_retry::connect_roster_retry;
use super::dial_directory_publishers::dial_directory_publishers;
use super::dial_known::dial_known;
use super::dialable::dialable;
use super::drive_roster::drive_roster;
use super::maps::{AttemptedMap, ConnectedMap, StaggerMap};
use super::publish_pre_get_union::publish_pre_get_union;
use super::resolve_room::resolve_room;
use super::run_config::RunConfig;
use super::run_context::RunContext;
use super::send_expected::send_expected;
use super::subscribe_topics::subscribe_topics;
use super::wait_ready::wait_ready;

pub async fn run(mut config: RunConfig) {
    let Some((peer_id, addrs)) = wait_ready(&mut config.ready_rx).await else {
        warn!(target: "room_lobby", "discovery: no libp2p ready signal, discovery disabled");
        return;
    };
    let peer_id = RemotePeerId(peer_id);
    let addrs = dialable(addrs);
    let ws_port = *config.endpoint;
    let mut directory = connect_catalog_retry(ws_port, &config.id).await;
    info!(target: "room_lobby", key = %directory.contract_key, "discovery: directory connected");
    let Some((room, room_params)) =
        resolve_room(&mut config, &mut directory, &peer_id, &addrs).await
    else {
        warn!(target: "room_lobby", "discovery: no room resolved, discovery disabled");
        return;
    };
    info!(target: "room_lobby", room = %room.as_str(), "discovery: room resolved");
    config.room_tx.send_replace(Some(room.clone()));
    publish_pre_get_union(&config.expected_tx, &directory.slots, &peer_id, &room);
    let roster =
        connect_roster_retry(ws_port, &room, &room_params, config.own, &peer_id, &addrs).await;
    info!(target: "room_lobby", key = %roster.contract_key, own = *config.own, "discovery: roster connected");
    send_expected(&config.expected_tx, &roster.slots, config.own);
    subscribe_topics(&config.net_tx, &config.id, &room);

    let mut attempted: AttemptedMap = HashMap::new();
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
        id: config.id,
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
    };
    drive_roster(&mut ctx).await;
}

// no test_usage necessary
