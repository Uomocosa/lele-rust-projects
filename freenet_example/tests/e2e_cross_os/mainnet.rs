use std::time::{Duration, Instant};

use freenet_example::testing::finish_record;
use freenet_example::testing::load_telegram_creds;
use freenet_example::testing::send_video_file;
use freenet_example::testing::start_record;
use freenet_example::testing::wakeup_screen;
use freenet_example::testing::{ReconcileEnv, connect_with_retry, spawn_node, tick_until_merged};

fn fmt_secs(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64())
}

const CLIP_SECS: u64 = 25;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "cross-host mainnet: needs CROSS_OS_KEY env + two hosts; run with --ignored --nocapture"]
async fn cross_host_mainnet() {
    let total_start = Instant::now();
    wakeup_screen::wakeup_screen();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,freenet_example=info,freenet_example=info".into()),
        )
        .try_init()
        .ok();
    let env = ReconcileEnv::from_env();
    let tmp = tempfile::tempdir().expect("tempdir");
    let spawn_start = Instant::now();
    let port = spawn_node(&tmp).await.expect("spawn node");
    let spawn_elapsed = spawn_start.elapsed();
    let wasm = include_bytes!("../../contract/global_counter_contract.wasm");
    let params = hex::encode(env.key.as_bytes());
    let connect_start = Instant::now();
    let mut client = connect_with_retry(port, wasm, params.as_bytes(), env.tag).await;
    let connect_elapsed = connect_start.elapsed();
    println!(
        "connected machine={} tag={} key={}",
        env.machine, env.tag, client.contract_key
    );
    let convergence_start = Instant::now();
    let (ticks, foreign_tags, count) = tick_until_merged(&mut client, env.deadline).await;
    let convergence_elapsed = convergence_start.elapsed();
    let recording_start = Instant::now();
    let clip_path = tmp.path().join("clip.mp4");
    let video = start_record::start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record::finish_record(child, &clip_path)
    });
    let recording_elapsed = recording_start.elapsed();
    let total_elapsed = total_start.elapsed();
    let record = serde_json::json!({
        "machine": env.machine,
        "own": env.tag,
        "count": count,
        "foreign_tags": foreign_tags,
        "ticks": ticks,
        "elapsed_secs": total_elapsed.as_secs_f64(),
    });
    std::fs::write(&env.log_path, format!("{record}\n")).expect("write log");
    println!("reconcile record: {record}");
    if let (Some(path), Some(creds)) = (video, load_telegram_creds::load_creds()) {
        let converged = !foreign_tags.is_empty();
        let caption = format!(
            "freenet_example cross-mainnet \u{b7} machine={} tag={} \u{b7} converged={converged} \u{b7} {}\n\
timings:\n\
\u{b7} spawn: {} s\n\
\u{b7} connect: {} s\n\
\u{b7} convergence: {} s\n\
\u{b7} recording: {} s\n\
\u{b7} total: {} s",
            env.machine,
            env.tag,
            env.key,
            fmt_secs(spawn_elapsed),
            fmt_secs(connect_elapsed),
            fmt_secs(convergence_elapsed),
            fmt_secs(recording_elapsed),
            fmt_secs(total_elapsed),
        );
        send_video_file::send_video_file(&creds, &path, &caption);
    }
    assert!(
        !foreign_tags.is_empty(),
        "machine={} tag={} never observed foreign slot within {}s (count={count}, ticks={ticks})",
        env.machine,
        env.tag,
        0
    );
}
