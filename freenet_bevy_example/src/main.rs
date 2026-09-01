use std::sync::Arc;
use std::time::Duration;

use bevy::DefaultPlugins;
use bevy::MinimalPlugins;
use bevy::app::App;
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use freenet_bevy::cli;
use freenet_bevy::clicker;
use freenet_bevy::freenet;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let cli = cli::Cli::parse();
    let mode = cli.mode;
    let p2p_port = cli.p2p_port;
    let contract_wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<clicker::Command>();
    let (evt_tx, evt_rx) = tokio::sync::mpsc::unbounded_channel::<clicker::Event>();

    tokio::spawn(async move {
        connect_and_run(p2p_port, contract_wasm, cmd_rx, evt_tx).await;
    });

    let config = clicker::Config::new(cmd_tx, evt_rx);

    match mode {
        cli::Mode::Gui => {
            App::new()
                .add_plugins(DefaultPlugins)
                .add_plugins(clicker::Plugin { config })
                .add_plugins(clicker::GuiPlugin)
                .run();
        }
        cli::Mode::Cli => {
            App::new()
                .add_plugins(MinimalPlugins)
                .add_plugins(clicker::Plugin { config })
                .add_plugins(clicker::CliPlugin)
                .run();
        }
    }
}

/// Starts the embedded node, deploys/connects to the counter contract, and then hands off to
/// `command_handler` for the rest of the process lifetime. Always publishes (deploy-if-missing)
/// against a self-contained embedded node — there is no remote/subscribe mode in this example.
async fn connect_and_run(
    p2p_port: u16,
    contract_wasm: Vec<u8>,
    cmd_rx: tokio::sync::mpsc::UnboundedReceiver<clicker::Command>,
    evt_tx: tokio::sync::mpsc::UnboundedSender<clicker::Event>,
) {
    // Held for the whole process: dropping it deletes the running node's config/data/log dirs.
    let _node_dir;
    let (node_host, node_port) = match start_embedded_node(p2p_port).await {
        Ok((host, port, dir)) => {
            _node_dir = dir;
            (host, port)
        }
        Err(e) => {
            evt_tx
                .send(clicker::Event::ConnectionError(format!(
                    "failed to start local node: {e}"
                )))
                .ok();
            return;
        }
    };

    let (client, contract_key, initial_count) =
        match setup_contract(&node_host, node_port, &contract_wasm).await {
            Ok(r) => r,
            Err(e) => {
                evt_tx
                    .send(clicker::Event::ConnectionError(format!(
                        "setup failed: {e}"
                    )))
                    .ok();
                return;
            }
        };

    info!(target: "freenet_bevy", key = %contract_key, count = initial_count, "connected, running");

    evt_tx
        .send(clicker::Event::Init {
            contract_key,
            count: initial_count,
        })
        .ok();

    command_handler(client, contract_key, cmd_rx, evt_tx).await;
}

// needed helper:
async fn recv_timeout(client: &mut freenet::FreenetClient) -> Result<HostResponse, String> {
    match client.recv_response_timeout(Duration::from_secs(60)).await {
        Some(Ok(r)) => Ok(r),
        Some(Err(e)) => Err(format!("{e}")),
        None => Err("timeout after 60s".into()),
    }
}

// needed helper: always publishes (deploy-if-missing, get-if-present).
async fn setup_contract(
    host: &str,
    port: u16,
    wasm: &[u8],
) -> Result<(freenet::FreenetClient, ContractKey, u64), String> {
    let mut client = loop {
        match freenet::FreenetClient::connect(host, port).await {
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
    let initial_count = loop {
        match recv_timeout(&mut client).await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                break bincode::deserialize(state.as_ref()).map_err(|e| format!("deser: {e}"))?;
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
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {
                continue;
            }
            other => return Err(format!("unexpected: {other:?}")),
        }
    };

    Ok((client, contract_key, initial_count))
}

// needed helper:
async fn command_handler(
    mut client: freenet::FreenetClient,
    contract_key: ContractKey,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<clicker::Command>,
    evt_tx: tokio::sync::mpsc::UnboundedSender<clicker::Event>,
) {
    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(clicker::Command::Increment { count }) => {
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
                                    evt_tx.send(clicker::Event::UpdateResponse { count }).ok();
                                    break;
                                }
                                Ok(HostResponse::ContractResponse(
                                    ContractResponse::UpdateNotification { update, .. },
                                )) => {
                                    let nc = clicker::count_from_update(&update);
                                    evt_tx.send(clicker::Event::Notification { count: nc }).ok();
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
                    let count = clicker::count_from_update(&update);
                    evt_tx.send(clicker::Event::Notification { count }).ok();
                }
            }
        }
    }
}

// needed helper:
/// Returns the `TempDir` so the caller can keep it alive: it backs the node's config, data and
/// log dirs, and dropping it deletes them out from under the still-running node.
async fn start_embedded_node(
    p2p_port: u16,
) -> Result<(String, u16, tempfile::TempDir), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;

    let listener =
        std::net::TcpListener::bind((std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();

    info!(target: "freenet_bevy", port, "starting in-process network-mode node");

    let ws_config = ::freenet::config::WebsocketApiConfig {
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = ::freenet::server::serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ::freenet::config::ConfigArgs {
        mode: Some(::freenet::local_node::OperationMode::Network),
        network_api: ::freenet::config::NetworkArgs {
            is_gateway: true,
            public_address: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(p2p_port),
            ..Default::default()
        },
        config_paths: ::freenet::config::ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = config_args.build().await?;
    let node_config = ::freenet::local_node::NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;

    tokio::spawn(async move {
        if let Err(e) = ::freenet::run_network_node(node).await {
            tracing::error!(target: "freenet_bevy", error = %e, "node exited with error");
        }
    });

    tokio::time::sleep(Duration::from_secs(20)).await;

    Ok(("127.0.0.1".to_string(), port, tmp))
}
// no test_usage necessary
