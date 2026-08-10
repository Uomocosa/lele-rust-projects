//! Coexistence spike: does a freenet node and a libp2p swarm actually run in the
//! same process, on the same tokio runtime, at the same time?
//!
//! Two independent progress signals are measured concurrently for `RUN_SECS`:
//!   * freenet — a websocket client asks the node's client API for a contract that
//!     does not exist; a `NotFound` (or any structured response) proves the node's
//!     event loop is live.
//!   * libp2p  — two swarms in this same process, one dialing the other over both
//!     TCP and QUIC, exchanging ping RTTs.
//!
//! QUIC is enabled on purpose: freenet's own transport is UDP, so this also checks
//! that the two do not fight over UDP sockets.

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, HostResponse};
use freenet_stdlib::prelude::ContractInstanceId;
use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, Swarm, SwarmBuilder, noise, ping, tcp, yamux};
use tracing::{info, warn};

const RUN_SECS: u64 = 60;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    // Held for the whole run: dropping it deletes the node's config/data/log dirs.
    let (node_host, node_port, _node_dir) = start_embedded_node().await?;
    info!(target: "spike", host = %node_host, port = node_port, "freenet node spawned");

    let (freenet_tx, mut freenet_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    tokio::spawn(probe_freenet(node_host, node_port, freenet_tx));

    let mut swarm_a = build_swarm()?;
    let mut swarm_b = build_swarm()?;
    let peer_a = *swarm_a.local_peer_id();
    let peer_b = *swarm_b.local_peer_id();
    info!(target: "spike", %peer_a, %peer_b, "libp2p swarms built");

    swarm_a.listen_on("/ip4/127.0.0.1/tcp/0".parse::<Multiaddr>()?)?;
    swarm_a.listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse::<Multiaddr>()?)?;

    let mut report = Report::new();
    let deadline = tokio::time::sleep(Duration::from_secs(RUN_SECS));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            msg = freenet_rx.recv() => match msg {
                Some(m) => {
                    info!(target: "spike", freenet = %m, "freenet probe");
                    report.freenet_replies += 1;
                    if report.freenet_first.is_none() {
                        report.freenet_first = Some(m.clone());
                    }
                    report.freenet_last = Some(m);
                }
                None => report.freenet_probe_ended = true,
            },
            event = swarm_a.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = &event {
                    let dial = address.clone().with_p2p(peer_a).unwrap_or_else(|a| a);
                    info!(target: "spike", %dial, "swarm A listening, dialing from B");
                    if let Err(e) = swarm_b.dial(dial) {
                        warn!(target: "spike", error = %e, "dial failed");
                    }
                }
                report.note("A", &event);
            },
            event = swarm_b.select_next_some() => report.note("B", &event),
        }
    }

    report.print(RUN_SECS);
    Ok(())
}

/// Both swarms get TCP *and* QUIC so the UDP-vs-freenet question is exercised.
// needed helper:
fn build_swarm() -> Result<Swarm<ping::Behaviour>, Box<dyn Error>> {
    let swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| {
            ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(2)))
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(RUN_SECS + 10)))
        .build();
    Ok(swarm)
}

/// Asks the node for a contract that cannot exist. Any structured reply means the
/// node's client API and event loop are running; repeats every 5s so we can tell
/// "alive once" from "alive throughout".
// needed helper:
async fn probe_freenet(host: String, port: u16, tx: tokio::sync::mpsc::UnboundedSender<String>) {
    let mut client = loop {
        match connect_ws(&host, port).await {
            Ok(c) => break c,
            Err(e) => {
                info!(target: "spike", error = %e, "ws connect failed, retrying in 1s");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };
    // Not counted as a readiness signal: the WS listener is bound before the node is
    // built, so a successful handshake says nothing about the node being live.
    info!(target: "spike", "websocket handshake ok");

    let bogus = ContractInstanceId::new([7u8; 32]);
    loop {
        let req = ClientRequest::ContractOp(ContractRequest::Get {
            key: bogus,
            return_contract_code: false,
            subscribe: false,
            blocking_subscribe: false,
        });
        // Timed here, around the request itself. Timing it in the receiving select
        // loop instead would measure when that loop got around to draining the
        // channel, which is not the same thing.
        let sent = std::time::Instant::now();
        match request(&mut client, req).await {
            Ok(resp) => {
                tx.send(format!("[{:?}] {resp}", sent.elapsed())).ok();
            }
            Err(e) => {
                tx.send(format!("probe error: {e}")).ok();
                return;
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

// needed helper:
async fn connect_ws(host: &str, port: u16) -> Result<Ws, Box<dyn Error + Send + Sync>> {
    let url = format!("ws://{host}:{port}/v1/contract/command?encodingProtocol=native");
    let mut request =
        <String as tokio_tungstenite::tungstenite::client::IntoClientRequest>::into_client_request(
            url,
        )?;
    request.headers_mut().insert(
        "encoding-protocol",
        http::HeaderValue::from_static("native"),
    );
    let (stream, _) = tokio_tungstenite::connect_async(request).await?;
    Ok(stream)
}

// needed helper:
async fn request(
    client: &mut Ws,
    req: ClientRequest<'_>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    use futures_util::{SinkExt, StreamExt as _};
    use tokio_tungstenite::tungstenite::Message;

    let bytes = bincode::serialize(&req)?;
    client.send(Message::Binary(bytes.into())).await?;

    let fut = async {
        while let Some(msg) = client.next().await {
            match msg? {
                Message::Binary(b) => {
                    let decoded: Result<HostResponse, _> = bincode::deserialize::<
                        Result<HostResponse, freenet_stdlib::client_api::ClientError>,
                    >(&b)?
                    .map_err(|e| e.to_string());
                    return Ok(match decoded {
                        Ok(r) => format!("{r:?}"),
                        Err(e) => format!("ClientError: {e}"),
                    });
                }
                Message::Ping(_) | Message::Pong(_) => continue,
                other => return Ok(format!("non-binary frame: {other:?}")),
            }
        }
        Err::<String, Box<dyn Error + Send + Sync>>("stream closed".into())
    };

    match tokio::time::timeout(Duration::from_secs(30), fut).await {
        Ok(r) => r,
        Err(_) => Err("timeout after 30s".into()),
    }
}

/// Same startup path as `freenet_bevy_example_2`, minus its hardcoded 20s sleep —
/// measuring how long readiness actually takes is part of the point.
// needed helper:
async fn start_embedded_node() -> Result<(String, u16, tempfile::TempDir), Box<dyn Error>> {
    let tmp = tempfile::tempdir()?;

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();

    let ws_config = freenet::config::WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = freenet::server::serve_client_api_with_listener(ws_config, listener).await?;

    let p2p_port = free_udp_port()?;
    let config_args = freenet::config::ConfigArgs {
        mode: Some(freenet::local_node::OperationMode::Network),
        network_api: freenet::config::NetworkArgs {
            is_gateway: true,
            public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(p2p_port),
            ..Default::default()
        },
        config_paths: freenet::config::ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = config_args.build().await?;
    let node_config = freenet::local_node::NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;

    info!(target: "spike", p2p_port, "freenet p2p (UDP) port");

    tokio::spawn(async move {
        if let Err(e) = freenet::run_network_node(node).await {
            tracing::error!(target: "spike", error = %e, "freenet node exited with error");
        }
    });

    Ok(("127.0.0.1".to_string(), port, tmp))
}

// needed helper:
fn free_udp_port() -> Result<u16, Box<dyn Error>> {
    let sock = std::net::UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    Ok(sock.local_addr()?.port())
}

struct Report {
    started: std::time::Instant,
    freenet_replies: usize,
    freenet_first: Option<String>,
    freenet_last: Option<String>,
    freenet_probe_ended: bool,
    listen_addrs: Vec<Multiaddr>,
    connections: Vec<PeerId>,
    ping_first: Option<Duration>,
    pings_ok: usize,
    pings_err: usize,
}

impl Report {
    fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
            freenet_replies: 0,
            freenet_first: None,
            freenet_last: None,
            freenet_probe_ended: false,
            listen_addrs: Vec::new(),
            connections: Vec::new(),
            ping_first: None,
            pings_ok: 0,
            pings_err: 0,
        }
    }
}

impl Report {
    fn note(&mut self, side: &str, event: &SwarmEvent<ping::Event>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!(target: "spike", side, %address, "listening");
                self.listen_addrs.push(address.clone());
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!(target: "spike", side, %peer_id, addr = %endpoint.get_remote_address(), "connected");
                self.connections.push(*peer_id);
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                warn!(target: "spike", side, error = %error, "outgoing connection error");
            }
            SwarmEvent::Behaviour(ping::Event { peer, result, .. }) => match result {
                Ok(rtt) => {
                    info!(target: "spike", side, %peer, ?rtt, "ping ok");
                    self.ping_first
                        .get_or_insert_with(|| self.started.elapsed());
                    self.pings_ok += 1;
                }
                Err(e) => {
                    warn!(target: "spike", side, %peer, error = %e, "ping failed");
                    self.pings_err += 1;
                }
            },
            _ => {}
        }
    }

    fn print(&self, secs: u64) {
        println!("\n=== coexistence spike: {secs}s ===");
        println!("freenet responses      : {}", self.freenet_replies);
        println!("libp2p  time-to-1st-ping: {:?}", self.ping_first);
        println!("freenet probe ended    : {}", self.freenet_probe_ended);
        println!(
            "freenet FIRST response : {}",
            self.freenet_first.as_deref().unwrap_or("<none>")
        );
        println!(
            "freenet last response  : {}",
            self.freenet_last.as_deref().unwrap_or("<none>")
        );
        println!("libp2p listen addrs    : {}", self.listen_addrs.len());
        for a in &self.listen_addrs {
            println!("  {a}");
        }
        println!("libp2p connections     : {}", self.connections.len());
        println!(
            "libp2p pings ok / err  : {} / {}",
            self.pings_ok, self.pings_err
        );
        let verdict = self.freenet_replies > 0 && self.pings_ok > 0;
        println!(
            "COEXISTENCE            : {}",
            if verdict { "YES" } else { "NO" }
        );
    }
}
// no test_usage necessary
