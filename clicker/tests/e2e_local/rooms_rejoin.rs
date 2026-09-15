use std::time::{Duration, Instant};

use clicker_lib::testing::{
    TerminalGuard, build_game, cleanup_stale, drive_random, poke, require_xterm, spawn_xterm,
    tile_three, wakeup_screen,
};

const TIMEOUT_SECS: u64 = 300;
const GAME_TITLES: [&str; 3] = ["clicker-1", "clicker-2", "clicker-3"];
const CLICKS_EACH: u32 = 15;
const LAG_AGREE_TIMEOUT_SECS: u64 = 60;

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

fn parse_number_after(stripped: &str, needle: &str) -> Option<f64> {
    let idx = stripped.find(needle)?;
    let start = idx.checked_add(needle.len())?;
    let rest = stripped.get(start..)?;
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest.get(..end)?.parse::<f64>().ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyncState {
    p1: u64,
    p2: u64,
    p3: u64,
    global: u64,
}

fn parse_last_sync(path: &std::path::Path) -> Option<SyncState> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut last = None;
    for line in content.lines() {
        let stripped = strip_ansi(line);
        if !stripped.contains("sync lobby=") {
            continue;
        }
        let (Some(p1), Some(p2), Some(p3), Some(global)) = (
            parse_number_after(&stripped, " p1=").map(|v| v as u64),
            parse_number_after(&stripped, " p2=").map(|v| v as u64),
            parse_number_after(&stripped, " p3=").map(|v| v as u64),
            parse_number_after(&stripped, " global=").map(|v| v as u64),
        ) else {
            continue;
        };
        last = Some(SyncState { p1, p2, p3, global });
    }
    last
}

fn tick_lines(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|content| {
            content
                .lines()
                .filter(|l| l.contains("tick lobby="))
                .count()
        })
        .unwrap_or_default()
}

fn resolved_count(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|content| content.matches("cursor resolved peer=").count())
        .unwrap_or_default()
}

fn room_joined(path: &std::path::Path, room: &str) -> bool {
    log_contains(path, &format!("lobby={room}"))
}

async fn wait_until(secs: u64, mut cond: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed().as_secs() < secs {
        if cond() {
            return true;
        }
        poke();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    cond()
}

fn spawn_tag(
    bin: &std::path::Path,
    dir: &std::path::Path,
    tag: u64,
    contract_params: &str,
    lobby: Option<&str>,
    create: bool,
    transport: &str,
    log_name: &str,
) -> Option<TerminalGuard> {
    let log = dir.join(log_name);
    match spawn_xterm(
        bin,
        "blackboard-v1",
        lobby,
        create,
        tag,
        contract_params,
        None,
        transport,
        &log,
    ) {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("spawn xterm {tag} ({log_name}): {e}");
            None
        }
    }
}

fn kill_guard(guard: &mut TerminalGuard) {
    if let Some(child) = guard.child.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

async fn drive_title(title: &str, clicks: u32) {
    eprintln!("drive_random {title} x{clicks}");
    let owned = title.to_string();
    let moved = owned.clone();
    match tokio::task::spawn_blocking(move || drive_random(&moved, clicks)).await {
        Ok(Ok(())) => eprintln!("drive_random {owned} done"),
        Ok(Err(err)) => eprintln!("drive_random {owned} failed: {err}"),
        Err(err) => eprintln!("drive_random {owned} join failed: {err}"),
    }
}

async fn agree_within(logs: &[std::path::PathBuf], minimum: u64, secs: u64) -> Option<SyncState> {
    let start = Instant::now();
    while start.elapsed().as_secs() < secs {
        let states: Vec<Option<SyncState>> = logs.iter().map(|l| parse_last_sync(l)).collect();
        if states.len() == logs.len()
            && let [Some(a), Some(b), Some(c)] = states.as_slice()
            && a == b
            && b == c
            && a.global == a.p1.saturating_add(a.p2).saturating_add(a.p3)
            && a.p1 >= minimum
            && a.p2 >= minimum
            && a.p3 >= minimum
        {
            return Some(*a);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    None
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs X + 3 clicker windows + public Freenet mainnet; run with --ignored --nocapture"]
async fn rooms_rejoin() {
    wakeup_screen();
    assert!(require_xterm().is_ok(), "xterm/xdotool/wmctrl missing");
    cleanup_stale();
    let mut bin = std::path::PathBuf::new();
    match build_game() {
        Ok(path) => bin = path,
        Err(e) => eprintln!("build clicker release: {e}"),
    }
    assert!(bin.exists(), "binary not found: {}", bin.display());

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let persist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("rooms-rejoin-{timestamp}"));
    assert!(
        std::fs::create_dir_all(&persist_dir).is_ok(),
        "create {}",
        persist_dir.display()
    );

    let room = format!("room-{timestamp}");
    let transport = std::env::var("CLICKER_TRANSPORT")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "both".to_string());
    let contract_params = std::env::var("CLICKER_CONTRACT_PARAMS")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_default();

    let mut guard1 = spawn_tag(
        &bin,
        &persist_dir,
        1,
        &contract_params,
        Some(&room),
        true,
        &transport,
        "instance-1.log",
    )
    .expect("spawn 1");
    let log1 = persist_dir.join("instance-1.log");
    assert!(
        wait_until(180, || log_contains(&log1, "discovery: roster connected")).await,
        "instance 1 never joined roster"
    );
    let mut guard2 = spawn_tag(
        &bin,
        &persist_dir,
        2,
        &contract_params,
        Some(&room),
        false,
        &transport,
        "instance-2.log",
    )
    .expect("spawn 2");
    let log2 = persist_dir.join("instance-2.log");
    assert!(
        wait_until(180, || room_joined(&log2, &room)).await,
        "instance 2 never joined room"
    );
    let mut guard3 = spawn_tag(
        &bin,
        &persist_dir,
        3,
        &contract_params,
        Some(&room),
        false,
        &transport,
        "instance-3.log",
    )
    .expect("spawn 3");
    let log3 = persist_dir.join("instance-3.log");
    assert!(
        wait_until(180, || room_joined(&log3, &room)).await,
        "instance 3 never joined room"
    );
    assert!(tile_three(GAME_TITLES).is_ok(), "tile game windows");

    let room_marker = format!("lobby={room}");
    let converged = wait_until(TIMEOUT_SECS, || {
        [&log1, &log2, &log3]
            .iter()
            .all(|log| log_contains(log, "tick lobby="))
            && [&log1, &log2, &log3]
                .iter()
                .all(|log| log_contains(log, &room_marker))
            && [&log1, &log2, &log3]
                .iter()
                .all(|log| log_contains(log, "accounting for remote owner="))
            && [&log1, &log2, &log3]
                .iter()
                .all(|log| resolved_count(log) >= 2)
    })
    .await;
    assert!(converged, "no converge: {}", persist_dir.display());

    for title in GAME_TITLES {
        drive_title(title, CLICKS_EACH).await;
    }
    let baseline = agree_within(
        &[log1.clone(), log2.clone(), log3.clone()],
        u64::from(CLICKS_EACH),
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await
    .expect("no baseline agreement");
    eprintln!(
        "baseline: p1={} p2={} p3={} global={}",
        baseline.p1, baseline.p2, baseline.p3, baseline.global
    );

    kill_guard(&mut guard3);
    tokio::time::sleep(Duration::from_secs(20)).await;
    assert!(
        tick_lines(&log1) > 0 && tick_lines(&log2) > 0,
        "room stalled after app3 left"
    );
    let before_rejoin = tick_lines(&log2);
    let mut guard3 = spawn_tag(
        &bin,
        &persist_dir,
        3,
        &contract_params,
        Some(&room),
        false,
        &transport,
        "instance-3-rejoin.log",
    )
    .expect("respawn 3");
    let log3b = persist_dir.join("instance-3-rejoin.log");
    assert!(
        wait_until(180, || log_contains(&log3b, "discovery: roster connected")).await,
        "instance 3 never rejoined roster"
    );
    assert!(tile_three(GAME_TITLES).is_ok(), "re-tile game windows");
    let restored = agree_within(
        &[log1.clone(), log2.clone(), log3b.clone()],
        u64::from(CLICKS_EACH),
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await
    .expect("no agreement after app3 rejoin");
    assert!(
        restored.p1 >= baseline.p1 && restored.p2 >= baseline.p2 && restored.p3 >= baseline.p3,
        "app3 lost distributed state: baseline={baseline:?} restored={restored:?}"
    );
    assert!(
        tick_lines(&log2) > before_rejoin,
        "room stalled while app3 rejoined"
    );
    eprintln!("app3 rejoined with state intact");

    kill_guard(&mut guard1);
    let creator_gone_ticks = tick_lines(&log2);
    tokio::time::sleep(Duration::from_secs(25)).await;
    assert!(
        tick_lines(&log2) > creator_gone_ticks,
        "room stalled after creator left: creator is not special, the room must survive"
    );
    let mut guard1 = spawn_tag(
        &bin,
        &persist_dir,
        1,
        &contract_params,
        Some(&room),
        false,
        &transport,
        "instance-1-rejoin.log",
    )
    .expect("respawn 1");
    let log1b = persist_dir.join("instance-1-rejoin.log");
    assert!(
        wait_until(180, || log_contains(&log1b, "discovery: roster connected")).await,
        "creator never rejoined roster"
    );
    assert!(tile_three(GAME_TITLES).is_ok(), "re-tile game windows");
    let final_state = agree_within(
        &[log1b.clone(), log2.clone(), log3b.clone()],
        u64::from(CLICKS_EACH),
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await
    .expect("no agreement after creator rejoin");
    assert!(
        final_state.p1 >= restored.p1
            && final_state.p2 >= restored.p2
            && final_state.p3 >= restored.p3,
        "creator rejoin broke the room: restored={restored:?} final={final_state:?}"
    );
    eprintln!(
        "creator rejoined as ordinary peer: p1={} p2={} p3={} global={}",
        final_state.p1, final_state.p2, final_state.p3, final_state.global
    );

    kill_guard(&mut guard1);
    kill_guard(&mut guard2);
    kill_guard(&mut guard3);
}
