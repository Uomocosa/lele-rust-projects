use std::time::{Duration, Instant};

use freenet_example_3::testing::finish_record;
use freenet_example_3::testing::load_telegram_creds;
use freenet_example_3::testing::send_video_file;
use freenet_example_3::testing::start_record;
use freenet_example_3::testing::terminal;
use freenet_example_3::testing::wakeup_screen;
use freenet_example_3::testing::{build_game, new_contract_params};

const INSTANCES: usize = 3;
const TIMEOUT_SECS: u64 = 300;
const CLIP_SECS: u64 = 25;

fn log_contains(path: &std::path::Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|s| s.contains(needle))
}

fn fmt_secs(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs public Freenet mainnet + 3 standalone nodes; run with --ignored --nocapture"]
async fn local_mainnet() {
    let total_start = Instant::now();
    wakeup_screen::wakeup_screen();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        terminal::require_xterm().expect("xterm not found — FAIL fast per spec");
    }
    let contract_params = new_contract_params();
    let build_start = Instant::now();
    let bin = build_game().expect("build freenet-example-3 release");
    let build_elapsed = build_start.elapsed();
    assert!(bin.exists(), "binary not found: {}", bin.display());

    let tmp = tempfile::tempdir().expect("tempdir");
    let spawn_start = Instant::now();
    let mut terms = Vec::new();
    for i in 0..INSTANCES {
        let log = tmp.path().join(format!("instance-{i}.log"));
        let tag = u64::try_from(i).unwrap_or(0) + 1;
        let prefix_len = 8.min(contract_params.len());
        #[allow(clippy::string_slice)]
        let prefix = &contract_params[..prefix_len];
        let title = format!("freenet-3-{tag}-{prefix}");
        #[allow(clippy::expect_used, clippy::unwrap_used)]
        let guard = terminal::spawn_xterm(&bin, &contract_params, tag, &log, &title)
            .unwrap_or_else(|e| panic!("spawn xterm failed: {e}"));
        terms.push(guard);
    }
    let titles: Vec<String> = terms.iter().map(|g| g.window_title.clone()).collect();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        terminal::tile_three([&titles[0], &titles[1], &titles[2]])
            .unwrap_or_else(|e| panic!("{e}"));
    }
    let spawn_elapsed = spawn_start.elapsed();

    let convergence_start = Instant::now();
    let mut converged = false;
    while convergence_start.elapsed().as_secs() < TIMEOUT_SECS {
        let ready = terms
            .iter()
            .all(|g| log_contains(&g.log, "connected, running indefinitely"));
        if ready && terms.iter().all(|g| log_contains(&g.log, "tick")) {
            converged = true;
            break;
        }
        wakeup_screen::poke();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    let convergence_elapsed = convergence_start.elapsed();

    let recording_start = Instant::now();
    let clip_path = tmp.path().join("clip.mp4");
    let video = start_record::start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record::finish_record(child, &clip_path)
    });
    let recording_elapsed = recording_start.elapsed();

    for mut g in terms {
        let _ = g.child.kill();
        let _ = g.child.wait();
    }

    let total_elapsed = total_start.elapsed();

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
        "freenet_example_3 local-mainnet \u{b7} 3 instances \u{b7} converged={converged} \u{b7} {contract_params}\n\
timings:\n\
\u{b7} build: {} s\n\
\u{b7} spawn: {} s\n\
\u{b7} convergence: {} s\n\
\u{b7} recording: {} s\n\
\u{b7} total: {} s",
        fmt_secs(build_elapsed),
        fmt_secs(spawn_elapsed),
        fmt_secs(convergence_elapsed),
        fmt_secs(recording_elapsed),
        fmt_secs(total_elapsed),
    );
    send_video_file::send_video_file(&creds, &path, &caption);

    assert!(
        converged,
        "local mainnet with contract_params={contract_params} and {INSTANCES} instances did not converge within {TIMEOUT_SECS}s"
    );
}
