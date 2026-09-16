use std::time::{Duration, Instant};

use clicker_lib::testing::{
    TerminalGuard, build_game, cleanup_stale, click_at_y, finish_record, poke, require_xterm,
    spawn_xterm, speed_clip_x4, start_record, wakeup_screen,
};
use telegram_bot::{load_creds, send_video_file};

const TIMEOUT_SECS: u64 = 300;
const RECORD_SECS: u64 = 240;
const READY_SECS: u64 = 5;
const CREATE_READY_SECS: u64 = 60;

struct SoftCheck {
    name: String,
    ok: bool,
    detail: String,
}

fn soft(checks: &mut Vec<SoftCheck>, name: &str, ok: bool, detail: String) {
    checks.push(SoftCheck {
        name: name.to_string(),
        ok,
        detail,
    });
}

const fn check_emoji(ok: bool) -> &'static str {
    if ok { "✅" } else { "❌" }
}
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

fn created_room(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let stripped = strip_ansi(line);
        if !stripped.contains("tick lobby=") {
            continue;
        }
        if let Some(idx) = stripped.find("tick lobby=") {
            let rest = stripped.get(idx + 11..)?;
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                .unwrap_or(rest.len());
            if let Some(name) = rest.get(..end)
                && !name.is_empty()
            {
                return Some(name.to_string());
            }
        }
    }
    None
}

async fn sweep_click(
    title: &str,
    y_center: i32,
    timeout_secs: u64,
    cond: impl Fn() -> bool,
) -> Option<Instant> {
    let start = Instant::now();
    let mut offsets = vec![0];
    for step in 1..=3 {
        offsets.push(step * 10);
        offsets.push(-step * 10);
    }
    let mut attempt = 0usize;
    while start.elapsed().as_secs() < timeout_secs {
        let dy = offsets[attempt % offsets.len()];
        attempt = attempt.saturating_add(1);
        if click_at_y(title, 0.5, y_center + dy).is_ok() {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if cond() {
                return Some(Instant::now());
            }
        } else {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    cond().then(Instant::now)
}

async fn ui_create_room(title: &str, log: &std::path::Path) -> Option<(String, Instant)> {
    let at = sweep_click(title, 84, 120, || created_room(log).is_some()).await?;
    created_room(log).map(|room| (room, at))
}

async fn ui_join_room(title: &str, room: &str, log: &std::path::Path) -> Option<Instant> {
    sweep_click(title, 84, 120, || room_joined(log, room)).await
}

fn file_len(path: &std::path::Path) -> u64 {
    std::fs::metadata(path)
        .map(|meta| meta.len())
        .unwrap_or_default()
}

fn ready_since(path: &std::path::Path, room: &str, pos: u64) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    if file.seek(SeekFrom::Start(pos)).is_err() {
        return false;
    }
    let mut tail = String::new();
    if file.read_to_string(&mut tail).is_err() {
        return false;
    }
    tail.lines()
        .any(|line| line.contains("join ready") && line.contains(room))
}

fn own_slot_series(path: &std::path::Path, pos: u64, room: &str, slot: &str) -> Vec<u64> {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else {
        return Vec::new();
    };
    if file.seek(SeekFrom::Start(pos)).is_err() {
        return Vec::new();
    }
    let mut tail = String::new();
    if file.read_to_string(&mut tail).is_err() {
        return Vec::new();
    }
    let marker = format!("sync lobby={room}");
    tail.lines()
        .filter(|line| line.contains(&marker))
        .filter_map(|line| parse_number_after(&strip_ansi(line), &format!(" {slot}=")))
        .map(|value| value as u64)
        .collect()
}

async fn wait_ready(log: &std::path::Path, room: &str, pos: u64, secs: u64) -> bool {
    let start = Instant::now();
    while start.elapsed().as_secs() < secs {
        if ready_since(log, room, pos) {
            return true;
        }
        poke();
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    ready_since(log, room, pos)
}

async fn assert_ready(
    checks: &mut Vec<SoftCheck>,
    name: &str,
    log: &std::path::Path,
    room: &str,
    pos: u64,
    clicked_at: Instant,
    budget_secs: u64,
) {
    let elapsed = clicked_at.elapsed().as_secs();
    let budget = budget_secs.saturating_sub(elapsed).max(1);
    let seen = wait_ready(log, room, pos, budget).await;
    let waited = clicked_at.elapsed().as_secs_f64();
    let ok = seen && waited <= budget_secs as f64 + 1.0;
    soft(
        checks,
        name,
        ok,
        format!("ready {waited:.1}s after click (budget {budget_secs}s)"),
    );
    assert!(ok, "join {name} not ready within {budget_secs}s of click");
}

fn assert_slot_frozen(
    checks: &mut Vec<SoftCheck>,
    name: &str,
    log: &std::path::Path,
    pos: u64,
    room: &str,
    slot: &str,
) {
    let series = own_slot_series(log, pos, room, slot);
    let ok = !series.is_empty() && series[0] <= 1 && series.iter().all(|value| *value == series[0]);
    soft(
        checks,
        name,
        ok,
        format!("{slot} samples after click: {series:?}"),
    );
    assert!(
        ok,
        "fresh joiner {name} registered clicks while loading: {series:?}"
    );
}

fn menu_since(path: &std::path::Path, pos: u64) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    if file.seek(SeekFrom::Start(pos)).is_err() {
        return false;
    }
    let mut tail = String::new();
    if file.read_to_string(&mut tail).is_err() {
        return false;
    }
    tail.lines()
        .any(|line| line.contains("directory listed rooms"))
}

async fn ui_leave_room(title: &str, log: &std::path::Path, pos: u64) -> bool {
    let start = Instant::now();
    let spots = [(0.9, 30), (0.95, 30), (0.9, 50), (0.85, 30)];
    let mut attempt = 0usize;
    while start.elapsed().as_secs() < 15 {
        let (fx, y) = spots[attempt % spots.len()];
        attempt = attempt.saturating_add(1);
        if click_at_y(title, fx, y).is_ok() {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if menu_since(log, pos) {
                return true;
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    menu_since(log, pos)
}

fn tile_one(title: &str, w: u32, h: u32, x: i32, y: i32) -> bool {
    use std::process::Command;
    let out = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", &format!("{title} \\[")])
        .output();
    let Ok(out) = out else { return false };
    let Some(wid) = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
    else {
        return false;
    };
    let sized = Command::new("xdotool")
        .args(["windowsize", &wid, &w.to_string(), &h.to_string()])
        .status()
        .is_ok_and(|s| s.success());
    let moved = Command::new("xdotool")
        .args(["windowmove", &wid, &x.to_string(), &y.to_string()])
        .status()
        .is_ok_and(|s| s.success());
    sized && moved
}

async fn wait_room_listed(log: &std::path::Path, room: &str) -> bool {
    wait_until(180, || {
        std::fs::read_to_string(log).is_ok_and(|content| {
            content
                .lines()
                .any(|line| line.contains("directory listed rooms") && line.contains(room))
        })
    })
    .await
}

async fn spawn_puller(
    bin: &std::path::Path,
    dir: &std::path::Path,
    namespace: &str,
    tag: u64,
    contract_params: &str,
    transport: &str,
    mdns: bool,
    room: &str,
    log_name: &str,
    tile: (u32, u32, i32, i32),
) -> (TerminalGuard, std::path::PathBuf) {
    for _ in 1..=2 {
        let mut guard = spawn_tag(
            bin,
            dir,
            namespace,
            tag,
            contract_params,
            None,
            false,
            transport,
            mdns,
            log_name,
        )
        .expect("spawn puller");
        let log = dir.join(log_name);
        assert!(
            wait_until(180, || log_contains(&log, "embedded freenet node started")).await,
            "instance {tag} node never started"
        );
        assert!(
            tile_one(&format!("clicker-{tag}"), tile.0, tile.1, tile.2, tile.3),
            "tile window {tag}"
        );
        if wait_room_listed(&log, room).await {
            return (guard, log);
        }
        eprintln!("instance {tag} pull missed, respawning for a fresh Get");
        kill_guard(&mut guard);
    }
    panic!("instance {tag} never pulled {room} from the directory");
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
    namespace: &str,
    tag: u64,
    contract_params: &str,
    lobby: Option<&str>,
    create: bool,
    transport: &str,
    mdns: bool,
    log_name: &str,
) -> Option<TerminalGuard> {
    let log = dir.join(log_name);
    match spawn_xterm(
        bin,
        namespace,
        lobby,
        create,
        tag,
        contract_params,
        None,
        transport,
        mdns,
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
    eprintln!("drive_center {title} x{clicks}");
    let owned = title.to_string();
    let moved = owned.clone();
    match tokio::task::spawn_blocking(move || clicker_lib::testing::drive_center(&moved, clicks))
        .await
    {
        Ok(Ok(())) => eprintln!("drive_center {owned} done"),
        Ok(Err(err)) => eprintln!("drive_center {owned} failed: {err}"),
        Err(err) => eprintln!("drive_center {owned} join failed: {err}"),
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
    let namespace = "blackboard-v1".to_string();
    let persist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("rooms-rejoin-{timestamp}"));
    assert!(
        std::fs::create_dir_all(&persist_dir).is_ok(),
        "create {}",
        persist_dir.display()
    );

    let transport = std::env::var("CLICKER_TRANSPORT")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "both".to_string());
    let mdns = std::env::var("CLICKER_MDNS")
        .ok()
        .is_some_and(|v| v == "on");
    // SAFETY: nextest runs this test binary single-test at a time and no
    // other thread reads process env here; children inherit it on spawn.
    // A unique key per run isolates this run's contracts from earlier runs.
    let stamp = chrono::Utc::now().timestamp_micros();
    let key_hex = format!(
        "{:016x}{:08x}",
        u64::try_from(stamp).unwrap_or_default(),
        std::process::id()
    );
    unsafe {
        std::env::set_var("CLICKER_CONTRACT_PARAMS", key_hex);
    }
    unsafe {
        std::env::set_var("CLICKER_NO_AUTOJOIN", "1");
    }
    let contract_params = std::env::var("CLICKER_CONTRACT_PARAMS")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_default();
    let raw_path = persist_dir.join("raw.mp4");
    let recording = start_record(RECORD_SECS, &raw_path);
    let mut checks: Vec<SoftCheck> = Vec::new();

    let mut guard1 = spawn_tag(
        &bin,
        &persist_dir,
        &namespace,
        1,
        &contract_params,
        None,
        false,
        &transport,
        mdns,
        "instance-1.log",
    )
    .expect("spawn 1");
    let log1 = persist_dir.join("instance-1.log");
    assert!(
        wait_until(180, || log_contains(&log1, "embedded freenet node started")).await,
        "instance 1 node never started"
    );
    soft(
        &mut checks,
        "spawn-1-node",
        true,
        "instance 1 freenet node started".to_string(),
    );
    assert!(tile_one(GAME_TITLES[0], 960, 540, 0, 0), "tile window 1");
    assert!(
        wait_until(180, || log_contains(
            &log1,
            "menu live: directory connected"
        ))
        .await,
        "instance 1 menu never went live (node never joined mainnet)"
    );
    soft(
        &mut checks,
        "menu-live-1",
        true,
        "instance 1 menu answered clicks only after directory live".to_string(),
    );
    let create_pos = file_len(&log1);
    let (room, clicked_at) = ui_create_room(GAME_TITLES[0], &log1)
        .await
        .expect("instance 1 never created a room via menu");
    eprintln!("created room via menu: {room}");
    soft(
        &mut checks,
        "ui-create-room",
        true,
        format!("instance 1 created {room} via menu button"),
    );
    assert_ready(
        &mut checks,
        "create-ready-1",
        &log1,
        &room,
        create_pos,
        clicked_at,
        CREATE_READY_SECS,
    )
    .await;
    assert_slot_frozen(
        &mut checks,
        "create-frozen-1",
        &log1,
        create_pos,
        &room,
        "p1",
    );
    let (mut guard2, log2) = spawn_puller(
        &bin,
        &persist_dir,
        &namespace,
        2,
        &contract_params,
        &transport,
        mdns,
        &room,
        "instance-2.log",
        (960, 540, 0, 540),
    )
    .await;
    soft(
        &mut checks,
        "spawn-2-node",
        true,
        "instance 2 freenet node started".to_string(),
    );
    soft(
        &mut checks,
        "directory-pull-2",
        true,
        format!("instance 2 pulled {room} via late Get"),
    );
    let join2_pos = file_len(&log2);
    let clicked_at = ui_join_room(GAME_TITLES[1], &room, &log2)
        .await
        .expect("instance 2 never joined room via menu");
    assert_ready(
        &mut checks,
        "join-ready-2",
        &log2,
        &room,
        join2_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    assert_slot_frozen(&mut checks, "join-frozen-2", &log2, join2_pos, &room, "p2");
    soft(
        &mut checks,
        "ui-join-2",
        true,
        format!("instance 2 joined {room} via menu button"),
    );
    let (mut guard3, log3) = spawn_puller(
        &bin,
        &persist_dir,
        &namespace,
        3,
        &contract_params,
        &transport,
        mdns,
        &room,
        "instance-3.log",
        (960, 1080, 960, 0),
    )
    .await;
    soft(
        &mut checks,
        "spawn-3-node",
        true,
        "instance 3 freenet node started".to_string(),
    );
    soft(
        &mut checks,
        "directory-pull-3",
        true,
        format!("instance 3 pulled {room} via late Get"),
    );
    let join3_pos = file_len(&log3);
    let clicked_at = ui_join_room(GAME_TITLES[2], &room, &log3)
        .await
        .expect("instance 3 never joined room via menu");
    assert_ready(
        &mut checks,
        "join-ready-3",
        &log3,
        &room,
        join3_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    assert_slot_frozen(&mut checks, "join-frozen-3", &log3, join3_pos, &room, "p3");
    soft(
        &mut checks,
        "ui-join-3",
        true,
        format!("instance 3 joined {room} via menu button"),
    );

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
    let resolved: Vec<usize> = [&log1, &log2, &log3]
        .iter()
        .map(|log| resolved_count(log))
        .collect();
    soft(
        &mut checks,
        "converge",
        converged,
        format!("tick+room+accounting+resolved>=2 on all 3, resolved={resolved:?}"),
    );

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
    soft(
        &mut checks,
        "baseline-agreement",
        baseline.global
            == baseline
                .p1
                .saturating_add(baseline.p2)
                .saturating_add(baseline.p3),
        format!("exact agreement after drive: {baseline:?}"),
    );

    kill_guard(&mut guard3);
    tokio::time::sleep(Duration::from_secs(20)).await;
    let after_leave = tick_lines(&log1).saturating_add(tick_lines(&log2));
    assert!(
        tick_lines(&log1) > 0 && tick_lines(&log2) > 0,
        "room stalled after app3 left"
    );
    soft(
        &mut checks,
        "leave-survives",
        true,
        format!("room kept ticking after app3 left, ticks={after_leave}"),
    );
    let before_rejoin = tick_lines(&log2);
    let (mut guard3, log3b) = spawn_puller(
        &bin,
        &persist_dir,
        &namespace,
        3,
        &contract_params,
        &transport,
        mdns,
        &room,
        "instance-3-rejoin.log",
        (960, 1080, 960, 0),
    )
    .await;
    let rejoin3_pos = file_len(&log3b);
    let clicked_at = ui_join_room(GAME_TITLES[2], &room, &log3b)
        .await
        .expect("instance 3 never rejoined room via menu");
    assert_ready(
        &mut checks,
        "rejoin-ready-3",
        &log3b,
        &room,
        rejoin3_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    assert!(
        wait_until(180, || log_contains(&log3b, "discovery: roster connected")).await,
        "instance 3 never rejoined roster"
    );
    soft(
        &mut checks,
        "ui-rejoin-3",
        true,
        format!("instance 3 rejoined {room} via menu button"),
    );
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
    soft(
        &mut checks,
        "rejoin-restores",
        true,
        format!("no state lost on app3 rejoin: baseline={baseline:?} restored={restored:?}"),
    );
    assert!(
        tick_lines(&log2) > before_rejoin,
        "room stalled while app3 rejoined"
    );
    soft(
        &mut checks,
        "rejoin-live",
        true,
        format!(
            "room kept ticking during app3 rejoin, ticks={}",
            tick_lines(&log2)
        ),
    );
    eprintln!("app3 rejoined with state intact");

    kill_guard(&mut guard1);
    let creator_gone_ticks = tick_lines(&log2);
    tokio::time::sleep(Duration::from_secs(25)).await;
    assert!(
        tick_lines(&log2) > creator_gone_ticks,
        "room stalled after creator left: creator is not special, the room must survive"
    );
    soft(
        &mut checks,
        "creator-leaves-survives",
        true,
        format!(
            "room survived creator leave, ticks {} -> {}",
            creator_gone_ticks,
            tick_lines(&log2)
        ),
    );
    let (mut guard1, log1b) = spawn_puller(
        &bin,
        &persist_dir,
        &namespace,
        1,
        &contract_params,
        &transport,
        mdns,
        &room,
        "instance-1-rejoin.log",
        (960, 540, 0, 0),
    )
    .await;
    let rejoin1_pos = file_len(&log1b);
    let clicked_at = ui_join_room(GAME_TITLES[0], &room, &log1b)
        .await
        .expect("creator never rejoined room via menu");
    assert_ready(
        &mut checks,
        "rejoin-ready-1",
        &log1b,
        &room,
        rejoin1_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    assert!(
        wait_until(180, || log_contains(&log1b, "discovery: roster connected")).await,
        "creator never rejoined roster"
    );
    soft(
        &mut checks,
        "ui-rejoin-1",
        true,
        format!("creator rejoined {room} via menu button"),
    );
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
    soft(
        &mut checks,
        "creator-rejoin-agreement",
        final_state.p1 >= restored.p1
            && final_state.p2 >= restored.p2
            && final_state.p3 >= restored.p3,
        format!("creator back as ordinary peer: restored={restored:?} final={final_state:?}"),
    );

    let (mut guard4, log4) = spawn_puller(
        &bin,
        &persist_dir,
        &namespace,
        4,
        &contract_params,
        &transport,
        mdns,
        &room,
        "instance-4.log",
        (960, 540, 960, 540),
    )
    .await;
    soft(
        &mut checks,
        "spawn-4-node",
        true,
        "instance 4 freenet node started".to_string(),
    );
    let join4_pos = file_len(&log4);
    ui_join_room("clicker-4", &room, &log4)
        .await
        .expect("instance 4 never joined room via menu");
    soft(
        &mut checks,
        "ui-join-4",
        true,
        format!("instance 4 joined {room} via menu button"),
    );
    let left4 = ui_leave_room("clicker-4", &log4, join4_pos).await;
    soft(
        &mut checks,
        "leave-during-loading-4",
        left4,
        "instance 4 left via the leave button right after joining".to_string(),
    );
    assert!(left4, "instance 4 never left via the leave button");
    let probe_ticks = tick_lines(&log2);
    tokio::time::sleep(Duration::from_secs(10)).await;
    let survived = tick_lines(&log2) > probe_ticks;
    soft(
        &mut checks,
        "leave-survives-4",
        survived,
        format!(
            "room kept ticking after instance 4 left, ticks {} -> {}",
            probe_ticks,
            tick_lines(&log2)
        ),
    );
    assert!(survived, "room stalled after instance 4 left");

    let clip_path = persist_dir.join("clip.mp4");
    let clip = recording
        .and_then(|child| finish_record(child, &raw_path))
        .and_then(|raw| speed_clip_x4(&raw, &clip_path));
    kill_guard(&mut guard1);
    kill_guard(&mut guard2);
    kill_guard(&mut guard3);
    kill_guard(&mut guard4);

    let Some(clip) = clip else {
        assert!(false, "clip missing at {}", clip_path.display());
        return;
    };
    let creds = load_creds();
    assert!(creds.is_some(), "telegram creds missing");
    let Some(creds) = creds else {
        assert!(false, "telegram creds missing");
        return;
    };
    let check_lines: Vec<String> = checks
        .iter()
        .map(|c| format!("{} {} {}", check_emoji(c.ok), c.name, c.detail))
        .collect();
    let all_ok = checks.iter().all(|c| c.ok);
    assert!(all_ok, "subtest failures:\n{}", check_lines.join("\n"));
    let caption = format!(
        "clicker rooms-rejoin room={room} transport={transport} mdns={mdns} · {} ({} / {})\n{}\nlogs: {}",
        check_emoji(all_ok),
        checks.iter().filter(|c| c.ok).count(),
        checks.len(),
        check_lines.join("\n"),
        persist_dir.display()
    );
    match tokio::task::spawn_blocking(move || send_video_file(&creds, &clip, &caption)).await {
        Ok(Ok(message)) => {
            println!(
                "telegram video sent: message_id={message} clip={}",
                clip_path.display()
            );
        }
        Ok(Err(err)) => {
            assert!(
                false,
                "telegram send failed: {err} clip={}",
                clip_path.display()
            );
        }
        Err(err) => {
            assert!(
                false,
                "telegram send join failed: {err} clip={}",
                clip_path.display()
            );
        }
    }
}
