use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use freenet_example::{ClickerClient, Role};

const WASM: &[u8] = include_bytes!("../contract/clicker_contract.wasm");

#[tokio::test(flavor = "multi_thread")]
async fn test_standalone_demo_works() {
    let tmp = tempfile::tempdir().unwrap();

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).unwrap();
    let port = listener.local_addr().unwrap().port();

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await.unwrap();

    let config_args = ConfigArgs {
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
    let config = config_args.build().await.unwrap();
    let node_config = NodeConfig::new(config).await.unwrap();
    let node = node_config.build(clients).await.unwrap();

    let _node = tokio::spawn(async move {
        let _ = run_network_node(node).await;
    });

    tokio::time::sleep(Duration::from_secs(3)).await;

    let mut publisher = ClickerClient::connect("127.0.0.1", port, WASM, Role::Publish)
        .await
        .unwrap();
    assert_eq!(publisher.count(), 0, "publisher initial count");

    for expected in 1..=3 {
        let count = publisher.tick().await.unwrap();
        assert_eq!(count, expected, "publisher tick {expected}");
    }

    let state = publisher.state().await.unwrap();
    assert_eq!(state, 3, "publisher final state");

    let mut subscriber = ClickerClient::connect("127.0.0.1", port, WASM, Role::Subscribe)
        .await
        .unwrap();
    let sub_state = subscriber.state().await.unwrap();
    assert_eq!(sub_state, 3, "subscriber reads state 3");
}
