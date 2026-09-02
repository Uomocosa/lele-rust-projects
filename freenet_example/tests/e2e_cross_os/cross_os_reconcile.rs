//! Cross-OS mainnet reconcile probe.
//!
//! `#[ignore]`d: driven per-machine by the self-hosted CI `cross-os-reconcile` job. Both
//! machines start a mainnet client node and a counter client with the SAME contract params
//! (from `CROSS_OS_KEY`) at the same time, tick for up to `CROSS_OS_DEADLINE_SECS`, and must
//! observe each other's slot through the network. Writes a JSONL record to `CROSS_OS_LOG`.

use freenet_example::testing::{ReconcileEnv, connect_with_retry, spawn_node, tick_until_merged};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs mainnet env (CROSS_OS_KEY)"]
async fn cross_os_reconcile() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,freenet_example=info,freenet_example=info".into()),
        )
        .init();
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
        "machine={} tag={} never observed a foreign slot through mainnet \
         within {}s (count={count}, ticks={ticks})",
        env.machine,
        env.tag,
        0
    );
    println!(
        "PASS machine={}: observed foreign slots {foreign_tags:?} after {}s",
        env.machine, 0
    );
}
