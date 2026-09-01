use std::time::{Duration, Instant};

use freenet_example_3::testing::finish_record;
use freenet_example_3::testing::load_telegram_creds;
use freenet_example_3::testing::send_text;
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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs public Freenet mainnet + 3 standalone nodes; run with --ignored --nocapture"]
async fn local_mainnet() {
    wakeup_screen::wakeup_screen();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        terminal::require_xterm().expect("xterm not found — FAIL fast per spec");
    }
    let contract_params = new_contract_params();
    let bin = build_game().expect("build freenet-example-3 release");
    assert!(bin.exists(), "binary not found: {}", bin.display());

    let tmp = tempfile::tempdir().expect("tempdir");
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

    let start = Instant::now();
    let mut converged = false;
    while start.elapsed().as_secs() < TIMEOUT_SECS {
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

    let clip_path = tmp.path().join("clip.mp4");
    let video = start_record::start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record::finish_record(child, &clip_path)
    });

    for mut g in terms {
        let _ = g.child.kill();
        let _ = g.child.wait();
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
