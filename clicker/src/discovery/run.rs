use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use freenet_libp2p_bevy_plugin::p2p;
use tracing::{info, warn};

use crate::clicker;
use crate::discovery;

const READY_TIMEOUT_SECS: u64 = 120;
const TICK_SECS: u64 = 1;
const ANNOUNCE_SECS: u64 = 30;
const WARMUP_SECS: u64 = 15;
const WARMUP_ANNOUNCE_SECS: u64 = 5;

pub async fn run(
    cmd_tx: tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    mut ready: tokio::sync::watch::Receiver<Option<(String, Vec<String>)>>,
    params: Vec<u8>,
    own: discovery::PlayerId,
) {
    let Some((peer_id, addrs)) = wait_ready(&mut ready).await else {
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
    let mut roster = loop {
        match discovery::connect_roster(
            "127.0.0.1",
            ws_port,
            clicker::contract_wasm(),
            &params,
            own,
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
    info!(target: "clicker", key = %roster.contract_key, own = *own, "discovery: roster connected");
    dial_known(&cmd_tx, &roster.slots, own);
    if roster.announce().is_err() {
        warn!(target: "clicker", "discovery: initial announce failed");
    }
    let mut last_announce = std::time::Instant::now();
    let started = std::time::Instant::now();
    loop {
        match roster.poll().await {
            Ok(fresh) => {
                for entry in fresh {
                    dial_entry(&cmd_tx, &entry);
                }
            }
            Err(e) => {
                warn!(target: "clicker", error = %e, "discovery: poll failed");
            }
        }
        let interval = if started.elapsed().as_secs() < WARMUP_SECS {
            WARMUP_ANNOUNCE_SECS
        } else {
            ANNOUNCE_SECS
        };
        if last_announce.elapsed().as_secs() >= interval {
            last_announce = std::time::Instant::now();
            if roster.announce().is_err() {
                warn!(target: "clicker", "discovery: announce failed");
            }
        }
        if roster.bridge_tick(std::time::Instant::now()).await.is_err() {
            warn!(target: "clicker", "discovery: bridge failed");
        }
        tokio::time::sleep(Duration::from_secs(TICK_SECS)).await;
    }
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
    slots: &discovery::RosterState,
    own: discovery::PlayerId,
) {
    for (id, entry) in slots {
        if *id == own {
            continue;
        }
        dial_entry(cmd_tx, entry);
    }
}

// needed helper: issues one libp2p dial for a discovered peer entry
fn dial_entry(
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Command<clicker::CursorMsg>>,
    entry: &discovery::PeerEntry,
) {
    if entry.addrs.is_empty() {
        return;
    }
    info!(target: "clicker", peer = %entry.peer_id, addrs = ?entry.addrs, "discovery: dialing peer");
    cmd_tx
        .send(p2p::Command::Dial {
            peer_id: String::new(),
            addrs: entry.addrs.clone(),
        })
        .ok();
}
// no test_usage necessary
