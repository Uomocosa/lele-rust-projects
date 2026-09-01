use std::fs::File;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use freenet_example_3::testing::finish_record;
use freenet_example_3::testing::load_telegram_creds;
use freenet_example_3::testing::send_text;
use freenet_example_3::testing::send_video_file;
use freenet_example_3::testing::start_record;
use freenet_example_3::testing::wakeup_screen;
use freenet_example_3::testing::{build_game, new_contract_params};

const INSTANCES: usize = 3;
const TIMEOUT_SECS: u64 = 300;
const CLIP_SECS: u64 = 25;

#[allow(clippy::expect_used)]
fn spawn_instance(
    bin: &std::path::Path,
    contract_params: &str,
    tag: u64,
    log_path: &std::path::Path,
) -> Child {
    let mut cmd = Command::new(bin);
    cmd.args([
        "--standalone",
        "--mainnet-client",
        "--contract-params",
        contract_params,
        "--instance-tag",
        &tag.to_string(),
    ])
    .env("RUST_LOG", "warn,freenet_example_3=info")
    .stdout(File::create(log_path).expect("create log"))
    .stderr(std::process::Stdio::piped());
    cmd.spawn().expect("spawn freenet-example-3")
}

fn log_contains(path: &std::path::Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|s| s.contains(needle))
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs public Freenet mainnet + 3 standalone nodes; run with --ignored --nocapture"]
async fn local_mainnet() {
    wakeup_screen::wakeup_screen();
    let contract_params = new_contract_params();
    let bin = build_game().expect("build freenet-example-3 release");
    assert!(bin.exists(), "binary not found: {}", bin.display());

    let tmp = tempfile::tempdir().expect("tempdir");
    let mut children: Vec<(Child, std::path::PathBuf)> = Vec::new();
    for i in 0..INSTANCES {
        let log = tmp.path().join(format!("instance-{i}.log"));
        let child = spawn_instance(
            &bin,
            &contract_params,
            u64::try_from(i).unwrap_or(0) + 1,
            &log,
        );
        children.push((child, log));
    }

    let start = Instant::now();
    let mut converged = false;
    while start.elapsed().as_secs() < TIMEOUT_SECS {
        let ready = children
            .iter()
            .all(|(_, log)| log_contains(log, "connected, running indefinitely"));
        if ready && children.iter().all(|(_, log)| log_contains(log, "tick")) {
            converged = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    wakeup_screen::wakeup_screen();
    let clip_path = tmp.path().join("clip.mp4");
    let video = start_record::start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record::finish_record(child, &clip_path)
    });

    for (mut child, _) in children {
        let _ = child.kill();
        let _ = child.wait();
    }

    let Some(path) = video else {
        panic!(
            "telegram video missing: clip not recorded at {clip_path:?} converged={converged} contract_params={contract_params}"
        );
    };
    let Some(creds) = load_telegram_creds::load_creds() else {
        panic!(
            "telegram creds missing: TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID not found — symlink freenet_example_3/.env -> ../deskctrl_mcp/.env (converged={converged} contract_params={contract_params})"
        );
    };
    let caption = format!(
        "freenet_example_3 local-mainnet · 3 instances · converged={converged} · {contract_params}"
    );
    send_video_file::send_video_file(&creds, &path, &caption);
    send_text::send_text(
        &creds,
        &format!(
            "freenet_example_3 local-mainnet done: converged={converged} contract_params={contract_params} instances={INSTANCES}"
        ),
    );

    assert!(
        converged,
        "local mainnet with contract_params={contract_params} and {INSTANCES} instances did not converge within {TIMEOUT_SECS}s"
    );
}
