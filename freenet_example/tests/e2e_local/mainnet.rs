use std::fmt::Write as FmtWrite;
use std::time::{Duration, Instant};

use freenet_example::testing::finish_record;
use freenet_example::testing::load_telegram_creds;
use freenet_example::testing::send_video_file;
use freenet_example::testing::start_record;
use freenet_example::testing::wakeup_screen;
use freenet_example::testing::{
    build_game, new_contract_params, poke, require_xterm, spawn_xterm, tile_three,
};

const INSTANCES: usize = 3;
const TIMEOUT_SECS: u64 = 300;
const CLIP_SECS: u64 = 25;
const ACCUMULATION_SECS: u64 = 30;
const ACCUMULATION_TOLERANCE_PCT: u64 = 10;

fn log_contains(path: &std::path::Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|s| s.contains(needle))
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for cc in chars.by_ref() {
                if cc == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

fn parse_last_count(path: &std::path::Path) -> Option<u64> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut last = None;
    for line in content.lines() {
        if !line.contains("tick") {
            continue;
        }
        let stripped = strip_ansi(line);
        if let Some(idx) = stripped.find("count=") {
            let Some(start) = idx.checked_add(6) else {
                continue;
            };
            let Some(rest) = stripped.get(start..) else {
                continue;
            };
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            let Some(num_str) = rest.get(..end) else {
                continue;
            };
            if let Ok(v) = num_str.parse::<u64>() {
                last = Some(v);
            }
        }
    }
    last
}

fn fmt_secs(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64())
}

#[allow(clippy::too_many_lines)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs public Freenet mainnet + 3 standalone nodes; run with --ignored --nocapture"]
async fn local_mainnet() {
    let total_start = Instant::now();
    wakeup_screen::wakeup_screen();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        require_xterm().expect("xterm not found — FAIL fast per spec");
    }
    let contract_params = new_contract_params();
    let build_start = Instant::now();
    let bin = build_game().expect("build freenet-example-3 release");
    let build_elapsed = build_start.elapsed();
    assert!(bin.exists(), "binary not found: {}", bin.display());

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let prefix_len = 8.min(contract_params.len());
    #[allow(clippy::string_slice)]
    let prefix = &contract_params[..prefix_len];
    let run_id = format!("{timestamp}-{prefix}");
    let persist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(&run_id);
    std::fs::create_dir_all(&persist_dir).expect("create persist_dir");
    let tmp_path = persist_dir.clone();
    let spawn_start = Instant::now();
    let mut terms = Vec::new();
    for i in 0..INSTANCES {
        let log = tmp_path.join(format!("instance-{i}.log"));
        let Ok(tag) = u64::try_from(i) else {
            continue;
        };
        let tag = tag.saturating_add(1);
        let prefix_len = 8.min(contract_params.len());
        #[allow(clippy::string_slice)]
        let prefix = &contract_params[..prefix_len];
        let title = format!("freenet-3-{tag}-{prefix}");
        #[allow(clippy::expect_used, clippy::unwrap_used)]
        let guard = spawn_xterm(&bin, &contract_params, tag, &log, &title)
            .unwrap_or_else(|e| panic!("spawn xterm failed: {e}"));
        terms.push(guard);
    }
    let titles: Vec<String> = terms.iter().map(|g| g.window_title.clone()).collect();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        tile_three([&titles[0], &titles[1], &titles[2]]).unwrap_or_else(|e| panic!("{e}"));
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
        poke();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    let convergence_elapsed = convergence_start.elapsed();

    let accumulation_start = Instant::now();
    let start_counts: Vec<Option<u64>> = terms.iter().map(|g| parse_last_count(&g.log)).collect();
    tokio::time::sleep(Duration::from_secs(ACCUMULATION_SECS)).await;
    let accumulation_elapsed = accumulation_start.elapsed();
    let end_counts: Vec<Option<u64>> = terms.iter().map(|g| parse_last_count(&g.log)).collect();
    let elapsed_secs = accumulation_elapsed.as_secs();
    let Ok(inst_u64) = u64::try_from(INSTANCES) else {
        panic!("INSTANCES conversion");
    };
    let expected_total = elapsed_secs.saturating_mul(inst_u64);
    let min_expected_per_instance =
        elapsed_secs.saturating_mul(100 - ACCUMULATION_TOLERANCE_PCT) / 100;
    let mut accumulation_detail = String::new();
    let mut per_instance_ok = true;
    for (i, (s, e)) in start_counts.iter().zip(end_counts.iter()).enumerate() {
        if let (Some(ss), Some(ee)) = (s, e) {
            let delta = ee.saturating_sub(*ss);
            let is_ok = delta.saturating_add(1) >= min_expected_per_instance;
            per_instance_ok &= is_ok;
            let _ = write!(
                accumulation_detail,
                " inst{i}: {ss}->{ee} delta={delta} expected>={min_expected_per_instance} ok={is_ok};"
            );
        } else {
            per_instance_ok = false;
            let _ = write!(
                accumulation_detail,
                " inst{i}: start={s:?} end={e:?} missing;"
            );
        }
    }
    let total_start_sum: u64 = start_counts.iter().filter_map(|c| *c).sum();
    let total_end_sum: u64 = end_counts.iter().filter_map(|c| *c).sum();
    let total_delta = total_end_sum.saturating_sub(total_start_sum);
    let min_total = expected_total.saturating_mul(100 - ACCUMULATION_TOLERANCE_PCT) / 100;
    let accumulation_ok = total_delta >= min_total;
    let _ = write!(
        accumulation_detail,
        " total {total_start_sum}->{total_end_sum} delta={total_delta} expected>={min_total} per_instance_ok={per_instance_ok}"
    );

    let recording_start = Instant::now();
    let clip_path = tmp_path.join("clip.mp4");
    let video = start_record::start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record::finish_record(child, &clip_path)
    });
    let recording_elapsed = recording_start.elapsed();

    for mut g in terms {
        if let Some(child) = g.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    let total_elapsed = total_start.elapsed();

    let Some(path) = video else {
        panic!(
            "telegram video missing: clip not recorded at {clip_path:?} converged={converged} contract_params={contract_params}"
        );
    };
    let Some(creds) = load_telegram_creds::load_creds() else {
        panic!(
            "telegram creds missing: TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID not found — symlink freenet_example/.env -> ../deskctrl_mcp/.env (converged={converged} contract_params={contract_params})"
        );
    };
    let params_preview = contract_params.chars().take(16).collect::<String>();
    let caption = format!(
        "freenet_example local-mainnet \u{b7} 3 instances \u{b7} converged={converged} accumulation_ok={accumulation_ok} \u{b7} {params_preview}..\n\
{accumulation_detail}\n\
timings:\n\
\u{b7} build: {} s\n\
\u{b7} spawn: {} s\n\
\u{b7} convergence: {} s\n\
\u{b7} accumulation: {} s ({}% gap)\n\
\u{b7} recording: {} s\n\
\u{b7} total: {} s\n\
logs: {}",
        fmt_secs(build_elapsed),
        fmt_secs(spawn_elapsed),
        fmt_secs(convergence_elapsed),
        fmt_secs(accumulation_elapsed),
        ACCUMULATION_TOLERANCE_PCT,
        fmt_secs(recording_elapsed),
        fmt_secs(total_elapsed),
        tmp_path.display(),
    );
    if tmp_path.exists() {
        println!("persisted logs: {}", tmp_path.display());
        for e in std::fs::read_dir(&tmp_path)
            .unwrap_or_else(|_| panic!("read_dir {}", tmp_path.display()))
            .flatten()
        {
            println!(" log: {}", e.path().display());
        }
    }
    send_video_file::send_video_file(&creds, &path, &caption);

    assert!(
        converged,
        "local mainnet with contract_params={contract_params} and {INSTANCES} instances did not converge within {TIMEOUT_SECS}s logs: {}",
        tmp_path.display()
    );
    assert!(
        accumulation_ok,
        "accumulation failed with 10% gap: {accumulation_detail} logs: {}",
        tmp_path.display()
    );
}
