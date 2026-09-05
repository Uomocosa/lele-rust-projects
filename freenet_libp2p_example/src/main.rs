use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::sync::OnceLock;
use std::time::Duration;

use clap::Parser;
use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use tracing::info;

use freenet_libp2p_example::discovery::Discovery;
use freenet_libp2p_example::frame::Frame;
use freenet_libp2p_example::frame_random_letter::random_letter;
use freenet_libp2p_example::frame_sign_frame::sign_frame;
use freenet_libp2p_example::identity_bridge::libp2p_keypair_from_seed::libp2p_keypair_from_seed;
use freenet_libp2p_example::identity_bridge::pubkey_from_signing::pubkey_from_signing;
use freenet_libp2p_example::identity_bridge::signing_key_from_seed::signing_key_from_seed;
use freenet_libp2p_example::relay::gossip_state::GossipState;
use freenet_libp2p_example::relay::new_behaviour::new_behaviour;
use libp2p::{Multiaddr, SwarmBuilder, noise, tcp, yamux};

#[derive(Parser, Debug)]
#[command(name = "freenet-libp2p-example")]
struct Args {
    #[arg(long, default_value = "demo-lobby")]
    lobby: String,
    #[arg(long, default_value_t = 0)]
    seed: u64,
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long)]
    p2p_port: Option<u16>,
    #[arg(long, default_value_t = false)]
    standalone: bool,
    #[arg(long, default_value_t = false)]
    mainnet_client: bool,
}

// needed helper:
fn args() -> &'static Args {
    static CELL: OnceLock<Args> = OnceLock::new();
    CELL.get_or_init(Args::parse)
}

// needed helper:
fn lobby() -> String {
    args().lobby.clone()
}

// needed helper:
fn seed() -> u64 {
    args().seed
}

// needed helper:
fn mainnet_client() -> bool {
    args().mainnet_client
}

// needed helper:
fn standalone_flag() -> bool {
    args().standalone
}

#[tokio::main]
async fn main() {
    let log_dir = std::env::var("FREENET_LOG_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let file_appender =
        tracing_appender::rolling::never(&log_dir, format!("freenet-libp2p-{}.log", seed()));
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    std::mem::forget(guard);
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();
    let use_standalone = standalone_flag() || args().host.is_none();
    let result = if use_standalone {
        run_standalone().await
    } else {
        run_client().await
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}

// needed helper:
async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let host = args()
        .host
        .clone()
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port = args().port.unwrap_or(7509);
    let mut connected = connect_with_retry(&host, port).await?;
    run_loop(&mut connected).await
}

// needed helper:
async fn run_standalone() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();
    info!(port, "starting in-process network-mode node");
    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;
    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: !mainnet_client(),
            public_address: (!mainnet_client()).then_some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            public_port: (!mainnet_client()).then_some(p2p_port()),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = config_args.build().await?;
    let node_config = NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error=%e, "node exited with error");
        }
    });
    let mut connected = connect_with_retry("127.0.0.1", port).await?;
    run_loop(&mut connected).await
}

// needed helper:
async fn connect_with_retry(
    host: &str,
    port: u16,
) -> Result<Discovery, Box<dyn std::error::Error>> {
    let wasm = include_bytes!("../contract/letter_contract.wasm").to_vec();
    let mut attempt: u64 = 0;
    loop {
        attempt = attempt.saturating_add(1);
        let lobby = lobby();
        match Discovery::connect(host, port, &wasm, &lobby).await {
            Ok(c) => return Ok(c),
            Err(e) => {
                tracing::warn!(error=%e, attempt, "discovery connect failed, retrying");
                let backoff = std::cmp::min(attempt.saturating_mul(2), 10);
                tokio::time::sleep(Duration::from_secs(backoff)).await;
            }
        }
    }
}

// needed helper:
async fn do_tick(
    discovery: &mut Discovery,
    gossip: &mut GossipState,
    next_seq: &mut u64,
    last_next: &mut u8,
    signing: &ed25519_dalek::SigningKey,
    pubkey: &[u8; 32],
    swarm: &mut libp2p::Swarm<freenet_libp2p_example::relay::behaviour::Behaviour>,
) {
    discovery.poll().await;
    discovery.bridge_tick(std::time::Instant::now()).await;
    for (seq, entry) in discovery.chain.clone() {
        let frame = Frame {
            seq,
            prev: entry.prev,
            next: entry.next,
            author: entry.author,
            sig: entry.sig,
        };
        if !gossip.seen.contains_key(&seq) {
            gossip.insert(frame);
            *last_next = entry.next;
            *next_seq = (*next_seq).max(seq.checked_add(1).unwrap_or(seq));
        }
    }
    let frame = sign_frame(signing, *next_seq, *last_next, random_letter());
    println!(
        "peer_data tick broadcast seq={} prev={} next={} author={:?} lobby={}",
        frame.seq,
        char::from(frame.prev),
        char::from(frame.next),
        pubkey,
        lobby()
    );
    info!(seq=frame.seq, prev=%char::from(frame.prev), next=%char::from(frame.next), author=?pubkey, "tick broadcast seq={} prev={} next={}", frame.seq, char::from(frame.prev), char::from(frame.next));
    discovery.publish_frame(&frame).await.ok();
    gossip.insert(frame.clone());
    // Freenet-only dial — no mDNS: send via request_response to each Freenet-discovered peer
    for (peer_pubkey, rec) in discovery.peers.clone() {
        if peer_pubkey == *pubkey {
            continue;
        }
        let Ok(peer_id) = libp2p::PeerId::from_bytes(&rec.peer_id) else {
            continue;
        };
        for addr_str in rec.addrs {
            let Ok(addr) = addr_str.parse::<Multiaddr>() else {
                continue;
            };
            swarm.add_peer_address(peer_id, addr.clone());
            // dial if not already connected — real Freenet discovery, not mDNS
            if swarm.is_connected(&peer_id) {
                let req = freenet_libp2p_example::relay::LetterRequest(frame.clone());
                let _ = swarm.behaviour_mut().send_request(&peer_id, req);
                tracing::debug!(peer=%peer_id, addr=%addr, seq=frame.seq, "dial via Freenet-discovered addr");
            } else {
                let _ = swarm.dial(addr.clone());
                tracing::debug!(peer=%peer_id, addr=%addr, "dial via Freenet-discovered addr");
            }
            break;
        }
    }
    *next_seq = next_seq.checked_add(1).unwrap_or(*next_seq);
    *last_next = frame.next;
}

// needed helper:
async fn run_loop(discovery: &mut Discovery) -> Result<(), Box<dyn std::error::Error>> {
    let seed_val = seed();
    let mut seed_bytes = [0u8; 32];
    if seed_val == 0 {
        getrandom::getrandom(&mut seed_bytes)?;
    } else {
        seed_bytes[0..8].copy_from_slice(&seed_val.to_le_bytes());
        seed_bytes[8] = 0xAB;
    }
    let signing = signing_key_from_seed(&seed_bytes);
    let pubkey = pubkey_from_signing(&signing);
    let Some(kp) = libp2p_keypair_from_seed(&seed_bytes) else {
        return Err("keypair generation failed".into());
    };
    let mut swarm = SwarmBuilder::with_existing_identity(kp)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
        .with_behaviour(|_| new_behaviour())?
        .build();
    let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    swarm.listen_on(addr)?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let mut gossip = GossipState::new();
    for (seq, entry) in discovery.chain.clone() {
        let frame = Frame {
            seq,
            prev: entry.prev,
            next: entry.next,
            author: entry.author,
            sig: entry.sig,
        };
        gossip.insert(frame);
    }
    let mut next_seq = discovery.next_seq();
    let last_next_initial = discovery.last_next();
    let mut last_next = if gossip.seen.is_empty() && next_seq == 0 {
        let frame = sign_frame(&signing, 0, 0, random_letter());
        println!(
            "peer_data genesis seq={} prev={} next={} lobby={} seed={}",
            frame.seq,
            char::from(frame.prev),
            char::from(frame.next),
            lobby(),
            seed_val
        );
        info!(seq=frame.seq, prev=%char::from(frame.prev), next=%char::from(frame.next), "genesis");
        discovery.publish_frame(&frame).await.ok();
        gossip.insert(frame.clone());
        next_seq = next_seq.checked_add(1).unwrap_or(next_seq);
        frame.next
    } else {
        last_next_initial
    };
    info!(
        lobby=%lobby(),
        seed=seed_val,
        key=%discovery.key,
        peers=discovery.peers.len(),
        chain=discovery.chain.len(),
        "connected, running indefinitely"
    );
    // publish our own PeerRecord for Freenet discovery (127.0.0.1 learned via contract is acceptable for same-host run)
    {
        let addrs: Vec<String> = swarm.listeners().map(ToString::to_string).collect();
        let peer_id_bytes = swarm.local_peer_id().to_bytes();
        let _ = discovery
            .publish_peer(pubkey, &signing, peer_id_bytes, addrs)
            .await;
    }
    let mut tick_interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                do_tick(discovery, &mut gossip, &mut next_seq, &mut last_next, &signing, &pubkey, &mut swarm).await;
            }
            () = freenet_libp2p_example::relay::drive_swarm::drive_swarm(&mut swarm, &mut gossip) => {
            }
        }
    }
}

// needed helper:
fn p2p_port() -> u16 {
    if let Some(port) = args().p2p_port {
        return port;
    }
    UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .and_then(|s| s.local_addr())
        .map_or(0, |a| a.port())
}

// no test_usage necessary
