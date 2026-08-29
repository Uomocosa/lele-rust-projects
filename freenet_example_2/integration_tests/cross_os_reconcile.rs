//! Cross-OS mainnet reconcile probe.
//!
//! `#[ignore]`d: driven per-machine by the self-hosted CI `cross-os-reconcile` job. Both
//! machines start a mainnet client node and a counter client with the SAME contract params
//! (from `CROSS_OS_KEY`) at the same time, tick for up to `CROSS_OS_DEADLINE_SECS`, and must
//! observe each other's slot through the network. Writes a JSONL record to `CROSS_OS_LOG`.

use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::time::{Duration, Instant};

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use freenet_example_2::ClickerClient;
use freenet_example_2::Role;

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn cross_os_reconcile() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,freenet_example=info,freenet_example_2=info".into()),
        )
        .init();
    let machine = std::env::var("CROSS_OS_MACHINE").unwrap_or_else(|_| "linux".into());
    let key = std::env::var("CROSS_OS_KEY").unwrap_or_else(|_| "cross-os-default".into());
    let log_path =
        std::env::var("CROSS_OS_LOG").unwrap_or_else(|_| "cross-os-reconcile.log".into());
    let deadline = Duration::from_secs(
        std::env::var("CROSS_OS_DEADLINE_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900),
    );
    let tag: u64 = match machine.as_str() {
        "windows" => 2,
        _ => 1,
    };

    let _tmp = tempfile::tempdir().expect("tempdir");
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).expect("bind ws port");
    let port = listener.local_addr().expect("local addr").port();
    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener)
        .await
        .expect("serve client api");
    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(_tmp.path().to_path_buf()),
            data_dir: Some(_tmp.path().to_path_buf()),
            log_dir: Some(_tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = config_args.build().await.expect("node config");
    let node_config = NodeConfig::new(config).await.expect("node config build");
    let node = node_config.build(clients).await.expect("node build");
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            eprintln!("node exited with error: {e}");
        }
    });

    let wasm = include_bytes!("../contract/clicker_contract.wasm");
    let params = hex::encode(key.as_bytes());
    let mut client = None;
    let mut attempt = 0u64;
    while client.is_none() {
        attempt += 1;
        match ClickerClient::connect_with_tag(
            "127.0.0.1",
            port,
            wasm,
            params.as_bytes(),
            Role::Publish,
            tag,
        )
        .await
        {
            Ok(c) => client = Some(c),
            Err(e) => {
                println!("connect attempt {attempt} failed: {e}");
                let backoff = std::cmp::min(attempt * 5, 30);
                tokio::time::sleep(Duration::from_secs(backoff)).await;
            }
        }
    }
    let mut client = client.expect("client");
    println!(
        "connected machine={machine} tag={tag} key={}",
        client.contract_key()
    );

    let start = Instant::now();
    let mut ticks = 0u64;
    loop {
        match client.tick().await {
            Ok(count) => {
                ticks += 1;
                println!(
                    "tick machine={machine} tag={tag} count={count} owns={} ticks={ticks}",
                    client.own()
                );
            }
            Err(e) => eprintln!("tick error: {e}"),
        }
        client.note_foreign_slots();
        if let Err(e) = client.bridge_tick().await {
            eprintln!("bridge error: {e}");
        }
        let merged = !client.foreign_tags().is_empty() && ticks >= 30;
        if merged || start.elapsed() >= deadline {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let foreign_tags: Vec<u64> = client.foreign_tags();
    let count = client.count();
    let record = serde_json::json!({
        "machine": machine,
        "own": tag,
        "count": count,
        "foreign_tags": foreign_tags,
        "ticks": ticks,
        "elapsed_secs": start.elapsed().as_secs(),
    });
    std::fs::write(&log_path, format!("{record}\n")).expect("write log");
    println!("reconcile record: {record}");

    assert!(
        !foreign_tags.is_empty(),
        "machine={machine} tag={tag} never observed a foreign slot through mainnet \
         within {}s (count={count}, ticks={ticks})",
        start.elapsed().as_secs()
    );
    println!(
        "PASS machine={machine}: observed foreign slots {foreign_tags:?} after {}s",
        start.elapsed().as_secs()
    );
}
