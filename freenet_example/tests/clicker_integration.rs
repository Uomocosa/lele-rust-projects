use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use freenet_example::{ClickerClient, FreenetClient, Role};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const WASM_PATH: &str = "contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm";

fn load_wasm() -> Vec<u8> {
    std::fs::read(WASM_PATH)
        .expect("contract WASM not found — build with: cargo make build-contract")
}

/// A running network-mode freenet node. Dropping the value stops it.
struct TestNode {
    _tmp: tempfile::TempDir,
    port: u16,
    _task: tokio::task::JoinHandle<()>,
}

impl TestNode {
    async fn start() -> Self {
        let _ = tracing_subscriber::fmt::try_init();

        let tmp = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).unwrap();
        let port = listener.local_addr().unwrap().port();

        tracing::info!(port, "starting network-mode node");

        let ws_config = WebsocketApiConfig {
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
            ..Default::default()
        };
        let clients = serve_client_api_with_listener(ws_config, listener)
            .await
            .unwrap();

        let args = ConfigArgs {
            mode: Some(OperationMode::Network),
            network_api: NetworkArgs {
                is_gateway: true,
                skip_load_from_network: true,
                public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                public_port: Some(31337),
                ..Default::default()
            },
            config_paths: ConfigPathsArgs {
                config_dir: Some(tmp.path().to_path_buf()),
                data_dir: Some(tmp.path().to_path_buf()),
                log_dir: Some(tmp.path().to_path_buf()),
            },
            ..Default::default()
        };
        let config = args.build().await.unwrap();
        let node_config = NodeConfig::new(config).await.unwrap();
        let node = node_config.build(clients).await.unwrap();
        let task = tokio::spawn(async move {
            if let Err(e) = run_network_node(node).await {
                tracing::error!(error = %e, "node exited with error");
            }
        });

        Self {
            _tmp: tmp,
            port,
            _task: task,
        }
    }

    fn port(&self) -> u16 {
        self.port
    }
}

async fn connect(port: u16) -> FreenetClient {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        match FreenetClient::connect("127.0.0.1", port).await {
            Ok(c) => return c,
            Err(e) => {
                if tokio::time::Instant::now() >= deadline {
                    panic!("could not connect within 15 s: {e}");
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

/// Deploy the clicker contract. Always GETs first — only PUTs if NotFound.
/// Returns the deterministic ContractKey.
async fn deploy(client: &mut FreenetClient, wasm: &[u8]) -> ContractKey {
    let code = Arc::new(ContractCode::from(wasm.to_vec()));
    let params = Parameters::from(Vec::new());
    let wrapped = WrappedContract::new(code, params);
    let key = wrapped.key;
    let instance_id = *key.id();

    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client
        .send(ClientRequest::ContractOp(get_req))
        .await
        .unwrap();

    loop {
        match client.recv_response().await.unwrap() {
            HostResponse::ContractResponse(ContractResponse::GetResponse { key, .. }) => {
                return key;
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => break,
            other => panic!("unexpected response to initial GET: {other:?}"),
        }
    }

    let put_req = ContractRequest::Put {
        contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
        state: WrappedState::new(bincode::serialize(&0u64).unwrap()),
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: true,
    };
    client
        .send(ClientRequest::ContractOp(put_req))
        .await
        .unwrap();

    match client.recv_response().await.unwrap() {
        HostResponse::ContractResponse(
            ContractResponse::PutResponse { key } | ContractResponse::SubscribeResponse { key, .. },
        ) => key,
        other => panic!("unexpected response to PUT: {other:?}"),
    }
}

/// Subscribe to a contract (GET + subscribe). Returns the current count.
async fn subscribe(client: &mut FreenetClient, key: ContractKey) -> u64 {
    let get_req = ContractRequest::Get {
        key: *key.id(),
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client
        .send(ClientRequest::ContractOp(get_req))
        .await
        .unwrap();
    loop {
        match client.recv_response().await.unwrap() {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                return bincode::deserialize(state.as_ref()).unwrap();
            }
            other => panic!("unexpected subscribe response: {other:?}"),
        }
    }
}

/// Read the current count via GET.
async fn get_count(client: &mut FreenetClient, key: ContractKey) -> u64 {
    let get_req = ContractRequest::Get {
        key: *key.id(),
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client
        .send(ClientRequest::ContractOp(get_req))
        .await
        .unwrap();
    loop {
        match client.recv_response().await.unwrap() {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                return bincode::deserialize(state.as_ref()).unwrap();
            }
            other => panic!("unexpected GET response: {other:?}"),
        }
    }
}

/// Send an UPDATE and wait for the response.
async fn update_count(client: &mut FreenetClient, key: ContractKey, count: u64) {
    let state = State::from(bincode::serialize(&count).unwrap());
    let req = ContractRequest::Update {
        key,
        data: UpdateData::State(state),
    };
    client.send(ClientRequest::ContractOp(req)).await.unwrap();
    match client.recv_response().await.unwrap() {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
        other => panic!("unexpected UPDATE response: {other:?}"),
    }
}

/// Try to receive an UpdateNotification within `timeout`.
async fn recv_notification(client: &mut FreenetClient, timeout: Duration) -> Option<u64> {
    match tokio::time::timeout(timeout, client.recv()).await {
        Ok(Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update,
            ..
        }))) => {
            let count = match &update {
                UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
                UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
                _ => 0,
            };
            Some(count)
        }
        _ => None,
    }
}

/// Drain any pending messages from the client's recv channel.
async fn drain(client: &mut FreenetClient) {
    while let Some(_) = client.recv_timeout(Duration::from_millis(50)).await {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_basic_lifecycle() {
    let node = TestNode::start().await;
    let wasm = load_wasm();
    let mut client = connect(node.port()).await;

    let key = deploy(&mut client, &wasm).await;

    let count = get_count(&mut client, key).await;
    assert_eq!(count, 0, "freshly deployed contract should have count 0");

    update_count(&mut client, key, 42).await;
    let count = get_count(&mut client, key).await;
    assert_eq!(count, 42, "state should be 42 after update");

    update_count(&mut client, key, 99).await;
    let count = get_count(&mut client, key).await;
    assert_eq!(count, 99, "state should be 99 after second update");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_one_publisher_two_subscribers() {
    let node = TestNode::start().await;
    let wasm = load_wasm();
    let mut pub_ = connect(node.port()).await;
    let mut sub_a = connect(node.port()).await;
    let mut sub_b = connect(node.port()).await;

    let key = deploy(&mut pub_, &wasm).await;

    let init_a = subscribe(&mut sub_a, key).await;
    assert_eq!(init_a, 0, "subscriber A initial state");

    let init_b = subscribe(&mut sub_b, key).await;
    assert_eq!(init_b, 0, "subscriber B initial state");

    // --- first update ---
    update_count(&mut pub_, key, 5).await;

    let (notif_a_1, notif_b_1) = tokio::join!(
        recv_notification(&mut sub_a, Duration::from_secs(10)),
        recv_notification(&mut sub_b, Duration::from_secs(10)),
    );

    match notif_a_1 {
        Some(c) => assert_eq!(c, 5, "subscriber A should receive notification of 5"),
        None => assert_eq!(
            get_count(&mut sub_a, key).await,
            5,
            "subscriber A GET fallback"
        ),
    }
    match notif_b_1 {
        Some(c) => assert_eq!(c, 5, "subscriber B should receive notification of 5"),
        None => assert_eq!(
            get_count(&mut sub_b, key).await,
            5,
            "subscriber B GET fallback"
        ),
    }

    // --- second update ---
    update_count(&mut pub_, key, 10).await;

    let (notif_a_2, notif_b_2) = tokio::join!(
        recv_notification(&mut sub_a, Duration::from_secs(10)),
        recv_notification(&mut sub_b, Duration::from_secs(10)),
    );
    match notif_a_2 {
        Some(c) => assert_eq!(c, 10, "subscriber A should receive notification of 10"),
        None => assert_eq!(
            get_count(&mut sub_a, key).await,
            10,
            "subscriber A GET fallback"
        ),
    }
    match notif_b_2 {
        Some(c) => assert_eq!(c, 10, "subscriber B should receive notification of 10"),
        None => assert_eq!(
            get_count(&mut sub_b, key).await,
            10,
            "subscriber B GET fallback"
        ),
    }

    // Final explicit verification
    assert_eq!(get_count(&mut sub_a, key).await, 10);
    assert_eq!(get_count(&mut sub_b, key).await, 10);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_two_publishers() {
    let node = TestNode::start().await;
    let wasm = load_wasm();
    let mut writer_a = connect(node.port()).await;
    let mut writer_b = connect(node.port()).await;
    let mut verifier = connect(node.port()).await;

    let key = deploy(&mut writer_a, &wasm).await;

    // Both writers subscribe so they can see each other's updates.
    subscribe(&mut writer_b, key).await;
    subscribe(&mut verifier, key).await;

    update_count(&mut writer_a, key, 3).await;

    let (_notif_a, notif_b, _) = tokio::join!(
        recv_notification(&mut writer_b, Duration::from_secs(10)),
        recv_notification(&mut verifier, Duration::from_secs(10)),
        drain(&mut writer_a),
    );
    assert_eq!(notif_b, Some(3), "verifier sees 3 after first update");

    update_count(&mut writer_b, key, 7).await;

    let (_notif_a_2, notif_v_2) = tokio::join!(
        recv_notification(&mut writer_a, Duration::from_secs(10)),
        recv_notification(&mut verifier, Duration::from_secs(10)),
    );
    assert_eq!(notif_v_2, Some(7), "verifier sees 7 after second update");

    assert_eq!(
        get_count(&mut verifier, key).await,
        7,
        "final state should be 7 (last write wins)"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_persistence() {
    let node = TestNode::start().await;
    let wasm = load_wasm();

    let key;
    {
        let mut client = connect(node.port()).await;
        key = deploy(&mut client, &wasm).await;
        update_count(&mut client, key, 5).await;
        assert_eq!(get_count(&mut client, key).await, 5);
    }

    tokio::time::sleep(Duration::from_secs(3)).await;

    {
        let mut client = connect(node.port()).await;
        assert_eq!(
            get_count(&mut client, key).await,
            5,
            "state persists after client disconnect"
        );
        update_count(&mut client, key, 8).await;
        assert_eq!(get_count(&mut client, key).await, 8);
    }

    tokio::time::sleep(Duration::from_secs(3)).await;

    {
        let mut client = connect(node.port()).await;
        assert_eq!(
            get_count(&mut client, key).await,
            8,
            "state persists after second disconnect"
        );
        update_count(&mut client, key, 10).await;
        assert_eq!(get_count(&mut client, key).await, 10);
    }
}

// ---------------------------------------------------------------------------
// ClickerClient tests (same code path as main.rs)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_client_publish_subscribe() {
    let node = TestNode::start().await;
    let wasm = load_wasm();

    let mut pub_ = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
        .await
        .unwrap();
    let init = pub_.state().await.unwrap();
    assert_eq!(init, 0, "publisher sees initial state 0");

    let count = pub_.tick().await.unwrap();
    assert_eq!(count, 1, "first tick increments to 1");

    let mut sub = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe)
        .await
        .unwrap();
    let sub_count = sub.state().await.unwrap();
    assert_eq!(
        sub_count, 1,
        "subscriber reads state 1 after publisher tick"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_clicker_client_multiple_ticks() {
    let node = TestNode::start().await;
    let wasm = load_wasm();

    let mut clicker = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
        .await
        .unwrap();

    for expected in 1..=5 {
        let count = clicker.tick().await.unwrap();
        assert_eq!(count, expected, "tick {expected}");
    }

    let state = clicker.state().await.unwrap();
    assert_eq!(state, 5, "state matches after 5 ticks");
}
