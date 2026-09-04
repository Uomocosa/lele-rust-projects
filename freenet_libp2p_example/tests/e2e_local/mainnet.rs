use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use freenet_libp2p_example::testing::{
    TerminalGuard, build_game, finish_record, load_creds, new_contract_params, poke, require_xterm,
    send_video_file, spawn_xterm, start_record, tile_three, wakeup_screen,
};

const INSTANCES: usize = 3;
const TIMEOUT_SECS: u64 = 180;
const CLIP_SECS: u64 = 20;
const ACCUMULATION_SECS: u64 = 20;

fn log_contains(path: &Path, needle: &str) -> bool {
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

fn parse_last_seq(path: &Path) -> Option<u64> {
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

fn count_broadcasts(path: &Path) -> usize {
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

// needed helper:
fn setup_persist_dir(lobby: &str) -> Result<(PathBuf, String), String> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let prefix = lobby.chars().take(8).collect::<String>();
    let run_id = format!("{timestamp}-{prefix}");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(&run_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("create persist_dir: {e}"))?;
    Ok((dir, prefix))
}

// needed helper:
fn spawn_terms(
    bin: &Path,
    lobby: &str,
    prefix: &str,
    dir: &Path,
) -> Result<Vec<TerminalGuard>, String> {
    let mut terms = Vec::new();
    for i in 0..INSTANCES {
        let log = dir.join(format!("instance-{i}.log"));
        let seed = u64::try_from(i.checked_add(1).unwrap_or(1)).unwrap_or(1);
        let title = format!("flp-{seed}-{prefix}");
        let guard =
            spawn_xterm(bin, lobby, seed, &log, &title).map_err(|e| format!("spawn xterm: {e}"))?;
        terms.push(guard);
    }
    let titles: Vec<String> = terms.iter().map(|g| g.window_title.clone()).collect();
    let first = titles.first().ok_or("missing title 0")?;
    let second = titles.get(1).ok_or("missing title 1")?;
    let third = titles.get(2).ok_or("missing title 2")?;
    tile_three([first.as_str(), second.as_str(), third.as_str()])?;
    Ok(terms)
}

// needed helper:
async fn wait_convergence(terms: &[TerminalGuard]) -> (bool, Duration) {
    let start = Instant::now();
    let mut converged = false;
    while start.elapsed().as_secs() < TIMEOUT_SECS {
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
    (converged, start.elapsed())
}

// needed helper:
async fn do_accumulation(
    terms: &[TerminalGuard],
) -> (Vec<Option<u64>>, Vec<Option<u64>>, Duration) {
    let start = Instant::now();
    let start_seqs: Vec<Option<u64>> = terms.iter().map(|g| parse_last_seq(&g.log)).collect();
    tokio::time::sleep(Duration::from_secs(ACCUMULATION_SECS)).await;
    let elapsed = start.elapsed();
    let end_seqs: Vec<Option<u64>> = terms.iter().map(|g| parse_last_seq(&g.log)).collect();
    (start_seqs, end_seqs, elapsed)
}

// needed helper:
fn build_peers_block(
    start_seqs: &[Option<u64>],
    end_seqs: &[Option<u64>],
    terms: &[TerminalGuard],
    expected_fast: u64,
) -> (String, bool) {
    let mut per_peer = Vec::new();
    let mut all_ok = true;
    for (i, (s, e)) in start_seqs.iter().zip(end_seqs.iter()).enumerate() {
        let Some(term) = terms.get(i) else { continue };
        let cnt_s = count_broadcasts(&term.log);
        if let (Some(ss), Some(ee)) = (s, e) {
            let delta = ee.saturating_sub(*ss);
            let ok = delta >= expected_fast;
            all_ok &= ok;
            let display_idx = i.checked_add(1).unwrap_or(1);
            per_peer.push(format!(
                "{display_idx}. inst{i}: seq {ss}->{ee} delta={delta} (start offset {ss} from convergence) broadcasts={cnt_s} {} {} {}",
                check(ok),
                check(ok),
                if ok { "PASS" } else { "FAIL" }
            ));
        } else {
            all_ok = false;
            let display_idx = i.checked_add(1).unwrap_or(1);
            per_peer.push(format!(
                "{display_idx}. inst{i}: start={s:?} end={e:?} {} FAIL",
                check(false)
            ));
        }
    }
    (per_peer.join("\n"), all_ok)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs 3 xterms + ffmpeg; run with --ignored --nocapture"]
async fn local_mainnet() -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    wakeup_screen();
    if require_xterm().is_err() {
        return Err("xterm not found".into());
    }
    let lobby = new_contract_params();
    let build_start = Instant::now();
    let bin = build_game().map_err(|e| format!("build freenet-libp2p-example: {e}"))?;
    let build_elapsed = build_start.elapsed();
    if !bin.exists() {
        return Err(format!("binary missing {}", bin.display()).into());
    }
    let (persist_dir, prefix) = setup_persist_dir(&lobby)?;
    let terms = spawn_terms(&bin, &lobby, &prefix, &persist_dir)?;
    let spawn_elapsed = Instant::now()
        .duration_since(total_start)
        .checked_sub(build_elapsed)
        .ok_or("checked_sub failed")?;
    let (converged, conv_elapsed) = wait_convergence(&terms).await;
    let (start_seqs, end_seqs, acc_elapsed) = do_accumulation(&terms).await;
    let expected_fast = ACCUMULATION_SECS.saturating_mul(8);
    let (peers_block, all_ok) = build_peers_block(&start_seqs, &end_seqs, &terms, expected_fast);
    let recording_start = Instant::now();
    let clip_path = persist_dir.join("clip.mp4");
    let video = start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record(child, &clip_path)
    });
    let recording_elapsed = recording_start.elapsed();
    let total_elapsed = total_start.elapsed();
    let Some(path) = video else {
        return Err(
            format!("video missing at {clip_path:?} converged={converged} lobby={lobby}").into(),
        );
    };
    let Some(creds) = load_creds() else {
        return Err(format!(
            "telegram creds missing — symlink .env -> ../deskctrl_mcp/.env converged={converged} lobby={lobby}"
        )
        .into());
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
    let read_dir = std::fs::read_dir(&persist_dir).map_err(|e| format!("read_dir failed: {e}"))?;
    for e in read_dir.flatten() {
        println!(" log: {}", e.path().display());
    }
    send_video_file(&creds, &path, &caption);
    if !converged {
        return Err(format!("did not converge in {TIMEOUT_SECS}s lobby={lobby}").into());
    }
    if !all_ok {
        return Err(format!("accumulation failed peers: {peers_block}").into());
    }
    Ok(())
}
