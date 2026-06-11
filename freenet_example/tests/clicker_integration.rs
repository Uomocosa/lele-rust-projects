use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, WebsocketApiConfig};
use freenet::local_node::{Executor, OperationMode};
use freenet::run_local_node;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use freenet_example::FreenetClient;

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_e2e() {
    let _ = tracing_subscriber::fmt::try_init();

    let contract_wasm = std::fs::read(
        "contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm",
    )
    .expect("contract WASM not found — build first with: cargo build --release --target wasm32-unknown-unknown -p clicker_contract");

    tracing::info!("building config");
    let temp_dir = tempfile::tempdir().unwrap();
    let args = ConfigArgs {
        mode: Some(OperationMode::Local),
        config_paths: ConfigPathsArgs {
            config_dir: Some(temp_dir.path().to_path_buf()),
            data_dir: Some(temp_dir.path().to_path_buf()),
            log_dir: Some(temp_dir.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = Arc::new(args.build().await.unwrap());
    tracing::info!("config built");

    let executor = Executor::from_config_local(config.clone()).await.unwrap();
    tracing::info!("executor created");

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 17510,
        ..Default::default()
    };
    let _node = tokio::spawn(async move {
        tracing::info!("starting local node");
        run_local_node(executor, ws_config).await.unwrap();
    });

    tracing::info!("waiting 2s for node startup");
    tokio::time::sleep(Duration::from_secs(2)).await;
    tracing::info!("waited, now connecting");

    let mut client = FreenetClient::connect("127.0.0.1", 17510)
        .await
        .expect("failed to connect to node");

    let contract_code = Arc::new(ContractCode::from(contract_wasm));
    let params = Parameters::from(Vec::new());
    let wrapped = WrappedContract::new(contract_code, params);
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    let initial_state = WrappedState::new(bincode::serialize(&0u64).unwrap());
    let related = RelatedContracts::default();

    let put_req = ContractRequest::Put {
        contract: container,
        state: initial_state,
        related_contracts: related,
        subscribe: true,
        blocking_subscribe: true,
    };
    client
        .send(ClientRequest::ContractOp(put_req))
        .await
        .unwrap();

    let contract_key = match client.recv().await.unwrap() {
        HostResponse::ContractResponse(response) => match response {
            ContractResponse::PutResponse { key }
            | ContractResponse::SubscribeResponse { key, .. } => key,
            other => panic!("unexpected response to put: {other:?}"),
        },
        other => panic!("unexpected response: {other:?}"),
    };

    let new_state = State::from(bincode::serialize(&1u64).unwrap());
    let update_req = ContractRequest::Update {
        key: contract_key,
        data: UpdateData::State(new_state),
    };
    client
        .send(ClientRequest::ContractOp(update_req))
        .await
        .unwrap();

    let response = client.recv().await.unwrap();
    match response {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
        HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {}
        other => panic!("unexpected update response: {other:?}"),
    }
}
