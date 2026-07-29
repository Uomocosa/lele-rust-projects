use std::sync::Arc;
use std::time::Duration;

use bevy::DefaultPlugins;
use bevy::MinimalPlugins;
use bevy::app::App;
use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use freenet_bevy::Role;
use freenet_bevy::clicker::cli::CliPlugin;
use freenet_bevy::clicker::gui::GuiPlugin;
use freenet_bevy::clicker::{ClickerCommand, ClickerConfig, ClickerEvent, ClickerPlugin};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let has_role = args.iter().any(|a| a == "--role");
    let mode = parse_mode();
    let role = parse_role();
    let contract_wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();

    let (node_host, node_port) = if has_role {
        let host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port: u16 = std::env::var("FREENET_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7509);
        (host, port)
    } else {
        match start_embedded_node().await {
            Ok((host, port)) => (host, port),
            Err(e) => {
                eprintln!("Error starting embedded node: {e}");
                return;
            }
        }
    };

    let (client, contract_key, initial_count) =
        match setup_contract(&node_host, node_port, &contract_wasm, role).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("setup failed: {e}");
                return;
            }
        };

    info!(target: "freenet_bevy", key = %contract_key, count = initial_count, "connected, running");

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<ClickerCommand>();
    let (evt_tx, evt_rx) = tokio::sync::mpsc::unbounded_channel::<ClickerEvent>();

    let cmd_key = contract_key;
    tokio::spawn(async move {
        command_handler(client, cmd_key, cmd_rx, evt_tx).await;
    });

    let config = ClickerConfig::new(cmd_tx, evt_rx, contract_key, initial_count);

    match mode {
        Mode::Gui => {
            App::new()
                .add_plugins(DefaultPlugins)
                .add_plugins(ClickerPlugin { config })
                .add_plugins(GuiPlugin)
                .run();
        }
        Mode::Cli => {
            App::new()
                .add_plugins(MinimalPlugins)
                .add_plugins(ClickerPlugin { config })
                .add_plugins(CliPlugin)
                .run();
        }
    }
}

async fn recv_timeout(client: &mut freenet_bevy::FreenetClient) -> Result<HostResponse, String> {
    match client.recv_response_timeout(Duration::from_secs(60)).await {
        Some(Ok(r)) => Ok(r),
        Some(Err(e)) => Err(format!("{e}")),
        None => Err("timeout after 60s".into()),
    }
}

async fn setup_contract(
    host: &str,
    port: u16,
    wasm: &[u8],
    role: Role,
) -> Result<(freenet_bevy::FreenetClient, ContractKey, u64), String> {
    let mut client = loop {
        match freenet_bevy::FreenetClient::connect(host, port).await {
            Ok(c) => break c,
            Err(_) => {
                info!(target: "freenet_bevy", "connect failed, retrying in 1s");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let code = Arc::new(ContractCode::from(wasm.to_vec()));
    let params = Parameters::from(Vec::new());
    let wrapped = WrappedContract::new(code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();

    let initial_count = match role {
        Role::Publish => {
            let get_req = ContractRequest::Get {
                key: instance_id,
                return_contract_code: false,
                subscribe: true,
                blocking_subscribe: true,
            };
            client
                .send(ClientRequest::ContractOp(get_req))
                .await
                .map_err(|e| format!("send get: {e}"))?;
            loop {
                match recv_timeout(&mut client).await? {
                    HostResponse::ContractResponse(ContractResponse::GetResponse {
                        state, ..
                    }) => {
                        break bincode::deserialize(state.as_ref())
                            .map_err(|e| format!("deser: {e}"))?;
                    }
                    HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                        let put_req = ContractRequest::Put {
                            contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
                            state: WrappedState::new(
                                bincode::serialize(&0u64).map_err(|e| format!("ser: {e}"))?,
                            ),
                            related_contracts: RelatedContracts::default(),
                            subscribe: true,
                            blocking_subscribe: false,
                        };
                        client
                            .send(ClientRequest::ContractOp(put_req))
                            .await
                            .map_err(|e| format!("send put: {e}"))?;
                        recv_timeout(&mut client).await?;
                        let get_req = ContractRequest::Get {
                            key: instance_id,
                            return_contract_code: false,
                            subscribe: true,
                            blocking_subscribe: false,
                        };
                        client
                            .send(ClientRequest::ContractOp(get_req))
                            .await
                            .map_err(|e| format!("send get2: {e}"))?;
                        match recv_timeout(&mut client).await? {
                            HostResponse::ContractResponse(ContractResponse::GetResponse {
                                state,
                                ..
                            }) => {
                                break bincode::deserialize(state.as_ref())
                                    .map_err(|e| format!("deser2: {e}"))?;
                            }
                            other => {
                                return Err(format!("unexpected after deploy: {other:?}"));
                            }
                        }
                    }
                    HostResponse::ContractResponse(ContractResponse::SubscribeResponse {
                        ..
                    }) => continue,
                    other => return Err(format!("unexpected: {other:?}")),
                }
            }
        }
        Role::Subscribe => loop {
            let get_req = ContractRequest::Get {
                key: instance_id,
                return_contract_code: false,
                subscribe: true,
                blocking_subscribe: true,
            };
            client
                .send(ClientRequest::ContractOp(get_req))
                .await
                .map_err(|e| format!("send sub: {e}"))?;
            match recv_timeout(&mut client).await? {
                HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                    break bincode::deserialize(state.as_ref())
                        .map_err(|e| format!("deser: {e}"))?;
                }
                _ => {
                    info!(
                        target: "freenet_bevy",
                        %instance_id,
                        "contract not found, retrying in 1s"
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        },
    };

    Ok((client, contract_key, initial_count))
}

async fn command_handler(
    mut client: freenet_bevy::FreenetClient,
    contract_key: ContractKey,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<ClickerCommand>,
    evt_tx: tokio::sync::mpsc::UnboundedSender<ClickerEvent>,
) {
    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(ClickerCommand::Increment { count }) => {
                        let data = State::from(bincode::serialize(&count).unwrap());
                        let req = ContractRequest::Update {
                            key: contract_key,
                            data: UpdateData::State(data),
                        };
                        if client.send(ClientRequest::ContractOp(req)).await.is_err() {
                            break;
                        }
                        loop {
                            match client.recv().await {
                                Ok(HostResponse::ContractResponse(
                                    ContractResponse::UpdateResponse { .. },
                                )) => {
                                    evt_tx.send(ClickerEvent::UpdateResponse { count }).ok();
                                    break;
                                }
                                Ok(HostResponse::ContractResponse(
                                    ContractResponse::UpdateNotification { update, .. },
                                )) => {
                                    let nc = count_from_update(&update);
                                    evt_tx.send(ClickerEvent::Notification { count: nc }).ok();
                                }
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        }
                    }
                    None => break,
                }
            }
            result = client.recv_timeout(Duration::from_millis(100)) => {
                if let Some(Ok(HostResponse::ContractResponse(
                    ContractResponse::UpdateNotification { update, .. },
                ))) = result
                {
                    let count = count_from_update(&update);
                    evt_tx.send(ClickerEvent::Notification { count }).ok();
                }
            }
        }
    }
}

fn count_from_update(update: &UpdateData) -> u64 {
    match update {
        UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
        UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
        _ => 0,
    }
}

async fn start_embedded_node() -> Result<(String, u16), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;

    let listener =
        std::net::TcpListener::bind((std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();

    info!(target: "freenet_bevy", port, "starting in-process network-mode node");

    let ws_config = WebsocketApiConfig {
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: true,
            public_address: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(p2p_port()),
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
            tracing::error!(target: "freenet_bevy", error = %e, "node exited with error");
        }
    });

    tokio::time::sleep(Duration::from_secs(20)).await;

    Ok(("127.0.0.1".to_string(), port))
}

fn p2p_port() -> u16 {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--p2p-port"
            && let Some(val) = args.next()
            && let Ok(port) = val.parse::<u16>()
        {
            return port;
        }
    }
    let socket =
        std::net::UdpSocket::bind((std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0))
            .expect("failed to probe an available UDP port");
    socket
        .local_addr()
        .expect("failed to read assigned port")
        .port()
}

enum Mode {
    Gui,
    Cli,
}

fn parse_mode() -> Mode {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--mode" {
            match args.next().as_deref() {
                Some("cli") => return Mode::Cli,
                Some("gui") => return Mode::Gui,
                _ => {}
            }
        }
    }
    Mode::Gui
}

fn parse_role() -> Role {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--role" {
            match args.next().as_deref() {
                Some("subscribe") => return Role::Subscribe,
                Some("publish") => return Role::Publish,
                _ => {}
            }
        }
    }
    Role::Publish
}
