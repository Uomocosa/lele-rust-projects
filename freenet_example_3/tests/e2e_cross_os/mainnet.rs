use freenet_example_3::testing::{ReconcileEnv, connect_with_retry, spawn_node, tick_until_merged};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "cross-host mainnet: needs CROSS_OS_KEY env + two hosts; run with --ignored --nocapture"]
async fn cross_host_mainnet() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,freenet_example=info,freenet_example_3=info".into()),
        )
        .try_init()
        .ok();
    let env = ReconcileEnv::from_env();
    let tmp = tempfile::tempdir().expect("tempdir");
    let port = spawn_node(&tmp).await.expect("spawn node");
    let wasm = include_bytes!("../../contract/clicker_contract.wasm");
    let params = hex::encode(env.key.as_bytes());
    let mut client = connect_with_retry(port, wasm, params.as_bytes(), env.tag).await;
    println!(
        "connected machine={} tag={} key={}",
        env.machine, env.tag, client.contract_key
    );
    let (ticks, foreign_tags, count) = tick_until_merged(&mut client, env.deadline).await;
    let record = serde_json::json!({
        "machine": env.machine,
        "own": env.tag,
        "count": count,
        "foreign_tags": foreign_tags,
        "ticks": ticks,
        "elapsed_secs": 0,
    });
    std::fs::write(&env.log_path, format!("{record}\n")).expect("write log");
    println!("reconcile record: {record}");
    assert!(
        !foreign_tags.is_empty(),
        "machine={} tag={} never observed foreign slot within {}s (count={count}, ticks={ticks})",
        env.machine,
        env.tag,
        0
    );
}
