use std::time::{Duration, Instant};

use freenet_libp2p_example::testing::{
    build_game, finish_record, load_creds, new_contract_params, poke, require_xterm,
    send_video_file, spawn_xterm, start_record, tile_three, wakeup_screen,
};

const INSTANCES: usize = 3;
const TIMEOUT_SECS: u64 = 180;
const CLIP_SECS: u64 = 20;
const ACCUMULATION_SECS: u64 = 20;

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

fn parse_last_seq(path: &std::path::Path) -> Option<u64> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut last = None;
    for line in content.lines() {
        if !line.contains("seq=") {
            continue;
        }
        let stripped = strip_ansi(line);
        if let Some(idx) = stripped.find("seq=") {
            let Some(start) = idx.checked_add(4) else {
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
            if let Ok(v) = num_str.parse::<u64>()
                && last.is_none_or(|p| v >= p)
            {
                last = Some(v);
            }
        }
    }
    last
}

fn count_broadcasts(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path).map_or(0, |s| {
        s.lines()
            .filter(|l| l.contains("broadcast") && l.contains("seq="))
            .count()
    })
}

fn fmt_secs(d: Duration) -> String {
    format!("{:.2}", d.as_secs_f64())
}

const fn check(ok: bool) -> &'static str {
    if ok { "✅" } else { "❌" }
}

#[allow(clippy::too_many_lines)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs 3 xterms + ffmpeg; run with --ignored --nocapture"]
async fn local_mainnet() {
    let total_start = Instant::now();
    wakeup_screen();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        require_xterm().expect("xterm not found");
    }
    let lobby = new_contract_params();
    let build_start = Instant::now();
    let bin = build_game().expect("build freenet-libp2p-example");
    let build_elapsed = build_start.elapsed();
    assert!(bin.exists(), "binary missing {}", bin.display());
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let prefix = lobby.chars().take(8).collect::<String>();
    let run_id = format!("{timestamp}-{prefix}");
    let persist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(&run_id);
    std::fs::create_dir_all(&persist_dir).expect("create persist_dir");
    let mut terms = Vec::new();
    for i in 0..INSTANCES {
        let log = persist_dir.join(format!("instance-{i}.log"));
        let seed = u64::try_from(i + 1).unwrap_or(1);
        let title = format!("flp-{seed}-{prefix}");
        #[allow(clippy::expect_used, clippy::unwrap_used)]
        let guard = spawn_xterm(&bin, &lobby, seed, &log, &title)
            .unwrap_or_else(|e| panic!("spawn xterm: {e}"));
        terms.push(guard);
    }
    let titles: Vec<String> = terms.iter().map(|g| g.window_title.clone()).collect();
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    {
        tile_three([&titles[0], &titles[1], &titles[2]]).unwrap_or_else(|e| panic!("{e}"));
    }
    let spawn_elapsed = Instant::now()
        .duration_since(total_start)
        .checked_sub(build_elapsed)
        .unwrap();
    let conv_start = Instant::now();
    let mut converged = false;
    while conv_start.elapsed().as_secs() < TIMEOUT_SECS {
        let ready = terms
            .iter()
            .all(|g| log_contains(&g.log, "broadcast") || log_contains(&g.log, "genesis"));
        if ready {
            converged = true;
            break;
        }
        poke();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    let conv_elapsed = conv_start.elapsed();
    let acc_start = Instant::now();
    let start_seqs: Vec<Option<u64>> = terms.iter().map(|g| parse_last_seq(&g.log)).collect();
    tokio::time::sleep(Duration::from_secs(ACCUMULATION_SECS)).await;
    let acc_elapsed = acc_start.elapsed();
    let end_seqs: Vec<Option<u64>> = terms.iter().map(|g| parse_last_seq(&g.log)).collect();
    let expected_fast = ACCUMULATION_SECS.saturating_mul(8);
    let mut per_peer = Vec::new();
    let mut all_ok = true;
    for (i, (s, e)) in start_seqs.iter().zip(end_seqs.iter()).enumerate() {
        let cnt_s = count_broadcasts(&terms[i].log);
        if let (Some(ss), Some(ee)) = (s, e) {
            let delta = ee.saturating_sub(*ss);
            let ok = delta >= expected_fast;
            all_ok &= ok;
            per_peer.push(format!(
                "{}. inst{i}: seq {ss}->{ee} delta={delta} (start offset {ss} from convergence) broadcasts={cnt_s} {} {}",
                i + 1,
                check(ok),
                if ok { "PASS" } else { "FAIL" }
            ));
        } else {
            all_ok = false;
            per_peer.push(format!(
                "{}. inst{i}: start={s:?} end={e:?} {} FAIL",
                i + 1,
                check(false)
            ));
        }
    }
    let peers_block = per_peer.join("\n");
    let recording_start = Instant::now();
    let clip_path = persist_dir.join("clip.mp4");
    let video = start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record(child, &clip_path)
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
        panic!("video missing at {clip_path:?} converged={converged} lobby={lobby}");
    };
    let Some(creds) = load_creds() else {
        panic!(
            "telegram creds missing — symlink .env -> ../deskctrl_mcp/.env converged={converged} lobby={lobby}"
        );
    };
    let caption = format!(
        "freenet_libp2p_example local-mainnet · {INSTANCES} instances · lobby `{lobby}` · {} converged={converged} · {} accumulation (FAST 100ms tick, expected≥{expected_fast} in {ACCUMULATION_SECS}s)\n\
{peers_block}\n\
timings:\n\
· build: {}s\n\
· spawn: {}s\n\
· convergence: {}s\n\
· accumulation: {}s\n\
· recording: {}s\n\
· total: {}s\n\
logs: {}\n\
contract: letter_contract fixed-lobby session-shard · gossip any-to-any · next truly random (no VRF)\n\
note: start seq 5→25 etc = 5 ticks during convergence (genesis+~5s poke) + 20 ticks in 20s at 1Hz; with 100ms tick expect ~200 delta (8/sec ×20s)",
        check(converged),
        check(all_ok),
        fmt_secs(build_elapsed),
        fmt_secs(spawn_elapsed),
        fmt_secs(conv_elapsed),
        fmt_secs(acc_elapsed),
        fmt_secs(recording_elapsed),
        fmt_secs(total_elapsed),
        persist_dir.display(),
    );
    println!("caption:\n{caption}");
    for e in std::fs::read_dir(&persist_dir).unwrap().flatten() {
        println!(" log: {}", e.path().display());
    }
    send_video_file(&creds, &path, &caption);
    assert!(
        converged,
        "did not converge in {TIMEOUT_SECS}s lobby={lobby}"
    );
    assert!(all_ok, "accumulation failed peers: {peers_block}");
}
