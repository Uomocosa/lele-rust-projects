use std::time::{Duration, Instant};

use clicker_lib::testing::{
    TerminalGuard, XtermSpec, build_game, cleanup_stale, click_at_y, finish_record, poke,
    require_xterm, spawn_xterm, speed_clip_x4, start_record, wakeup_screen,
};
use telegram_bot::{TestLog, send_video_best_effort};

const TIMEOUT_SECS: u64 = 300;
const RECORD_SECS: u64 = 240;
const READY_SECS: u64 = 45;
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

fn ensure(cond: bool, msg: String) -> Result<(), String> {
    if cond { Ok(()) } else { Err(msg) }
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

fn parse_u64_after(stripped: &str, needle: &str) -> Option<u64> {
    let idx = stripped.find(needle)?;
    let start = idx.checked_add(needle.len())?;
    let rest = stripped.get(start..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest.get(..end)?.parse::<u64>().ok()
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
            parse_u64_after(&stripped, " p1="),
            parse_u64_after(&stripped, " p2="),
            parse_u64_after(&stripped, " p3="),
            parse_u64_after(&stripped, " global="),
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

fn parse_owners(path: &std::path::Path) -> std::collections::BTreeSet<u64> {
    let mut owners = std::collections::BTreeSet::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return owners;
    };
    for line in content.lines() {
        let stripped = strip_ansi(line);
        if stripped.contains("sync lobby=") {
            owners.clear();
            continue;
        }
        if !stripped.contains("tick lobby=") {
            continue;
        }
        if let Some(owner) = parse_u64_after(&stripped, " owner=") {
            owners.insert(owner);
        }
    }
    owners
}

async fn wait_owners(path: &std::path::Path, expected: usize, secs: u64) -> bool {
    let start = Instant::now();
    while start.elapsed().as_secs() < secs {
        if parse_owners(path).len() == expected {
            return true;
        }
        poke();
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    parse_owners(path).len() == expected
}

async fn assert_owners(
    checks: &mut Vec<SoftCheck>,
    name: &str,
    log: &std::path::Path,
    expected: usize,
    secs: u64,
) {
    let ok = wait_owners(log, expected, secs).await;
    let seen = parse_owners(log).len();
    soft(
        checks,
        name,
        ok,
        format!("live owners {seen} (expected {expected})"),
    );
    assert_eq!(seen, expected, "{name}: wrong live player count");
}

async fn assert_all_owners(
    checks: &mut Vec<SoftCheck>,
    prefix: &str,
    logs: &[std::path::PathBuf],
    expected: usize,
    secs: u64,
) {
    for (index, log) in logs.iter().enumerate() {
        let name = format!("{prefix}-{}", index.saturating_add(1));
        assert_owners(checks, &name, log, expected, secs).await;
    }
}

async fn assert_rejoin_visible(
    checks: &mut Vec<SoftCheck>,
    name: &str,
    survivor_log: &std::path::Path,
    logs: &[std::path::PathBuf],
    secs: u64,
) {
    soft(
        checks,
        name,
        log_contains(survivor_log, "join reveal"),
        "a survivor logged the shared reveal after the rejoin".to_string(),
    );
    assert_all_owners(
        checks,
        &format!("{name}-owners"),
        logs,
        GAME_TITLES.len(),
        secs,
    )
    .await;
}

async fn await_rejoin(
    run: &mut Run<'_>,
    label: &str,
    rejoining: &std::path::Path,
    survivor: &std::path::Path,
) -> Result<SyncState, String> {
    ensure(
        wait_until(180, || {
            log_contains(rejoining, "discovery: roster connected")
        })
        .await,
        format!("{label}: never rejoined the roster"),
    )?;
    soft(
        &mut run.checks,
        &format!("ui-{label}"),
        true,
        format!("{label} rejoined {} via menu button", run.room),
    );
    let logs = [run.log1.clone(), run.log2.clone(), run.log3.clone()];
    assert_rejoin_visible(
        &mut run.checks,
        label,
        survivor,
        &logs,
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await;
    agree_within(&logs, u64::from(CLICKS_EACH), LAG_AGREE_TIMEOUT_SECS)
        .await
        .ok_or_else(|| format!("{label}: no agreement after rejoin"))
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
            let rest = stripped.get(idx.checked_add(11)?..)?;
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

const BRP_PORT_BASE: u16 = 17400;

fn brp_port(tag: u64) -> u16 {
    BRP_PORT_BASE.saturating_add(u16::try_from(tag).unwrap_or_default())
}

async fn brp_click(title: &str, port: u16, needle: &str) -> Result<(), String> {
    let title = title.to_string();
    let needle = needle.to_string();
    tokio::task::spawn_blocking(move || clicker_lib::testing::click_button(&title, port, &needle))
        .await
        .map_err(|err| format!("brp click join: {err}"))?
}

fn is_fresh_room(room: &str) -> bool {
    let Some(stamp) = room.strip_prefix("room-") else {
        return false;
    };
    let Ok(epoch) = stamp.parse::<u64>() else {
        return false;
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    now.saturating_sub(epoch) < 300
}

async fn ui_create_room(
    title: &str,
    log: &std::path::Path,
    port: u16,
) -> Option<(String, Instant)> {
    let start = Instant::now();
    while start.elapsed().as_secs() < 120 {
        if brp_click(title, port, "create-new-room").await.is_ok()
            && wait_until(30, || created_room(log).is_some()).await
        {
            return created_room(log).map(|room| (room, Instant::now()));
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    created_room(log).map(|room| (room, Instant::now()))
}

async fn ui_join_room(
    title: &str,
    room: &str,
    log: &std::path::Path,
    port: u16,
) -> Option<Instant> {
    let needle = format!("room:{room}");
    let start = Instant::now();
    while start.elapsed().as_secs() < 120 {
        if brp_click(title, port, &needle).await.is_ok() {
            let clicked_at = Instant::now();
            if room_joined(log, room) || wait_until(30, || room_joined(log, room)).await {
                return Some(clicked_at);
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    None
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
        .filter_map(|line| parse_u64_after(&strip_ansi(line), &format!(" {slot}=")))
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
    let waited_secs = clicked_at.elapsed().as_secs();
    let ok = seen && waited_secs <= budget_secs.saturating_add(1);
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
    let ok = series
        .first()
        .is_some_and(|first| *first == 0 && series.iter().all(|value| value == first));
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
        let slot = attempt.checked_rem(spots.len()).unwrap_or(0);
        let (fx, y) = spots.get(slot).copied().unwrap_or((0.9, 30));
        attempt = attempt.saturating_add(1);
        let clicked = click_at_y(title, fx, y).is_ok();
        tokio::time::sleep(Duration::from_secs(if clicked { 2 } else { 1 })).await;
        if clicked && menu_since(log, pos) {
            return true;
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

struct SpawnRequest<'a> {
    bin: &'a std::path::Path,
    dir: &'a std::path::Path,
    namespace: &'a str,
    tag: u64,
    contract_params: &'a str,
    lobby: Option<&'a str>,
    create: bool,
    transport: &'a str,
    mdns: bool,
    log_name: &'a str,
}

struct PullerRequest<'a> {
    bin: &'a std::path::Path,
    dir: &'a std::path::Path,
    namespace: &'a str,
    tag: u64,
    contract_params: &'a str,
    transport: &'a str,
    mdns: bool,
    room: &'a str,
    log_name: &'a str,
    tile: (u32, u32, i32, i32),
}

async fn spawn_puller(req: &PullerRequest<'_>) -> Option<(TerminalGuard, std::path::PathBuf)> {
    for _ in 1..=2 {
        let Some(mut guard) = spawn_tag(&SpawnRequest {
            bin: req.bin,
            dir: req.dir,
            namespace: req.namespace,
            tag: req.tag,
            contract_params: req.contract_params,
            lobby: None,
            create: false,
            transport: req.transport,
            mdns: req.mdns,
            log_name: req.log_name,
        }) else {
            continue;
        };
        let log = req.dir.join(req.log_name);
        assert!(
            wait_until(180, || log_contains(&log, "embedded freenet node started")).await,
            "instance {} node never started",
            req.tag
        );
        assert!(
            tile_one(
                &format!("clicker-{}", req.tag),
                req.tile.0,
                req.tile.1,
                req.tile.2,
                req.tile.3
            ),
            "tile window {}",
            req.tag
        );
        if wait_room_listed(&log, req.room).await {
            return Some((guard, log));
        }
        eprintln!(
            "instance {} pull missed, respawning for a fresh Get",
            req.tag
        );
        kill_guard(&mut guard);
    }
    None
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

fn spawn_tag(req: &SpawnRequest<'_>) -> Option<TerminalGuard> {
    let log = req.dir.join(req.log_name);
    let spec = XtermSpec {
        bin: req.bin,
        namespace: req.namespace,
        lobby: req.lobby,
        create: req.create,
        tag: req.tag,
        contract_params: req.contract_params,
        since_epoch: None,
        transport: req.transport,
        mdns: req.mdns,
        brp_port: Some(brp_port(req.tag)),
        log: &log,
    };
    match spawn_xterm(&spec) {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("spawn xterm {} ({}): {e}", req.tag, req.log_name);
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

struct Run<'a> {
    bin: &'a std::path::Path,
    persist_dir: &'a std::path::Path,
    namespace: &'a str,
    contract_params: &'a str,
    transport: &'a str,
    mdns: bool,
    checks: Vec<SoftCheck>,
    room: String,
    log1: std::path::PathBuf,
    log2: std::path::PathBuf,
    log3: std::path::PathBuf,
    log4: std::path::PathBuf,
    guard1: Option<TerminalGuard>,
    guard2: Option<TerminalGuard>,
    guard3: Option<TerminalGuard>,
    guard4: Option<TerminalGuard>,
    baseline: Option<SyncState>,
    restored: Option<SyncState>,
}

fn kill_opt(guard: &mut Option<TerminalGuard>) {
    if let Some(g) = guard.as_mut() {
        kill_guard(g);
    }
    *guard = None;
}

fn build_binary_for_rejoin() -> std::path::PathBuf {
    wakeup_screen();
    assert!(require_xterm().is_ok(), "xterm/xdotool/wmctrl missing");
    cleanup_stale();
    let bin = match build_game() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("build clicker release: {e}");
            std::path::PathBuf::new()
        }
    };
    assert!(bin.exists(), "binary not found: {}", bin.display());
    bin
}

fn prepare_persist_dir() -> std::path::PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let persist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("rooms-rejoin-{timestamp}"));
    assert!(
        std::fs::create_dir_all(&persist_dir).is_ok(),
        "create {}",
        persist_dir.display()
    );
    persist_dir
}

fn env_params() -> (String, String, String, bool) {
    let namespace = "blackboard-v1".to_string();
    let transport = std::env::var("CLICKER_TRANSPORT")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "both".to_string());
    let mdns = std::env::var("CLICKER_MDNS").is_ok_and(|v| v == "on");
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
    (namespace, contract_params, transport, mdns)
}

async fn phase_creator(run: &mut Run<'_>) -> Result<(), String> {
    let Some(guard) = spawn_tag(&SpawnRequest {
        bin: run.bin,
        dir: run.persist_dir,
        namespace: run.namespace,
        tag: 1,
        contract_params: run.contract_params,
        lobby: None,
        create: false,
        transport: run.transport,
        mdns: run.mdns,
        log_name: "instance-1.log",
    }) else {
        return Err("spawn 1".to_string());
    };
    run.guard1 = Some(guard);
    ensure(
        wait_until(180, || {
            log_contains(&run.log1, "embedded freenet node started")
        })
        .await,
        "instance 1 node never started".to_string(),
    )?;
    soft(
        &mut run.checks,
        "spawn-1-node",
        true,
        "instance 1 freenet node started".to_string(),
    );
    ensure(
        tile_one(GAME_TITLES[0], 960, 540, 0, 0),
        "tile window 1".to_string(),
    )?;
    ensure(
        wait_until(180, || {
            log_contains(&run.log1, "menu live: directory connected")
        })
        .await,
        "instance 1 menu never went live (node never joined mainnet)".to_string(),
    )?;
    soft(
        &mut run.checks,
        "menu-live-1",
        true,
        "instance 1 menu answered clicks only after directory live".to_string(),
    );
    let create_pos = file_len(&run.log1);
    let Some((room, clicked_at)) = ui_create_room(GAME_TITLES[0], &run.log1, brp_port(1)).await
    else {
        return Err("instance 1 never created a room via menu".to_string());
    };
    ensure(
        is_fresh_room(&room),
        format!("create button joined a stale room instead of creating one: {room}"),
    )?;
    eprintln!("created room via menu: {room}");
    soft(
        &mut run.checks,
        "ui-create-room",
        true,
        format!("instance 1 created {room} via menu button"),
    );
    assert_ready(
        &mut run.checks,
        "create-ready-1",
        &run.log1,
        &room,
        create_pos,
        clicked_at,
        CREATE_READY_SECS,
    )
    .await;
    assert_slot_frozen(
        &mut run.checks,
        "create-frozen-1",
        &run.log1,
        create_pos,
        &room,
        "p1",
    );
    run.room = room;
    Ok(())
}

async fn phase_join_2(run: &mut Run<'_>) -> Result<(), String> {
    let Some((guard, log)) = spawn_puller(&PullerRequest {
        bin: run.bin,
        dir: run.persist_dir,
        namespace: run.namespace,
        tag: 2,
        contract_params: run.contract_params,
        transport: run.transport,
        mdns: run.mdns,
        room: &run.room,
        log_name: "instance-2.log",
        tile: (960, 540, 0, 540),
    })
    .await
    else {
        return Err("spawn puller 2".to_string());
    };
    run.guard2 = Some(guard);
    run.log2 = log;
    soft(
        &mut run.checks,
        "spawn-2-node",
        true,
        "instance 2 freenet node started".to_string(),
    );
    soft(
        &mut run.checks,
        "directory-pull-2",
        true,
        format!("instance 2 pulled {} via late Get", run.room),
    );
    let join_pos = file_len(&run.log2);
    let Some(clicked_at) = ui_join_room(GAME_TITLES[1], &run.room, &run.log2, brp_port(2)).await
    else {
        return Err("instance 2 never joined room via menu".to_string());
    };
    assert_ready(
        &mut run.checks,
        "join-ready-2",
        &run.log2,
        &run.room,
        join_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    assert_slot_frozen(
        &mut run.checks,
        "join-frozen-2",
        &run.log2,
        join_pos,
        &run.room,
        "p2",
    );
    soft(
        &mut run.checks,
        "ui-join-2",
        true,
        format!("instance 2 joined {} via menu button", run.room),
    );
    Ok(())
}

async fn phase_join_3(run: &mut Run<'_>) -> Result<(), String> {
    let Some((guard, log)) = spawn_puller(&PullerRequest {
        bin: run.bin,
        dir: run.persist_dir,
        namespace: run.namespace,
        tag: 3,
        contract_params: run.contract_params,
        transport: run.transport,
        mdns: run.mdns,
        room: &run.room,
        log_name: "instance-3.log",
        tile: (960, 1080, 960, 0),
    })
    .await
    else {
        return Err("spawn puller 3".to_string());
    };
    run.guard3 = Some(guard);
    run.log3 = log;
    soft(
        &mut run.checks,
        "spawn-3-node",
        true,
        "instance 3 freenet node started".to_string(),
    );
    soft(
        &mut run.checks,
        "directory-pull-3",
        true,
        format!("instance 3 pulled {} via late Get", run.room),
    );
    let join_pos = file_len(&run.log3);
    let Some(clicked_at) = ui_join_room(GAME_TITLES[2], &run.room, &run.log3, brp_port(3)).await
    else {
        return Err("instance 3 never joined room via menu".to_string());
    };
    assert_ready(
        &mut run.checks,
        "join-ready-3",
        &run.log3,
        &run.room,
        join_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    assert_slot_frozen(
        &mut run.checks,
        "join-frozen-3",
        &run.log3,
        join_pos,
        &run.room,
        "p3",
    );
    soft(
        &mut run.checks,
        "ui-join-3",
        true,
        format!("instance 3 joined {} via menu button", run.room),
    );
    Ok(())
}

async fn phase_converge_drive(run: &mut Run<'_>) -> Result<(), String> {
    let room_marker = format!("lobby={}", run.room);
    let converged = wait_until(TIMEOUT_SECS, || {
        [&run.log1, &run.log2, &run.log3]
            .iter()
            .all(|log| log_contains(log, "tick lobby="))
            && [&run.log1, &run.log2, &run.log3]
                .iter()
                .all(|log| log_contains(log, &room_marker))
            && [&run.log1, &run.log2, &run.log3]
                .iter()
                .all(|log| log_contains(log, "accounting for remote owner="))
            && [&run.log1, &run.log2, &run.log3]
                .iter()
                .all(|log| resolved_count(log) >= 2)
    })
    .await;
    ensure(
        converged,
        format!("no converge: {}", run.persist_dir.display()),
    )?;
    let resolved: Vec<usize> = [&run.log1, &run.log2, &run.log3]
        .iter()
        .map(|log| resolved_count(log))
        .collect();
    soft(
        &mut run.checks,
        "converge",
        converged,
        format!("tick+room+accounting+resolved>=2 on all 3, resolved={resolved:?}"),
    );
    let converge_logs = [run.log1.clone(), run.log2.clone(), run.log3.clone()];
    assert_all_owners(
        &mut run.checks,
        "owners",
        &converge_logs,
        GAME_TITLES.len(),
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await;
    for title in GAME_TITLES {
        drive_title(title, CLICKS_EACH).await;
    }
    let baseline = agree_within(
        &[run.log1.clone(), run.log2.clone(), run.log3.clone()],
        u64::from(CLICKS_EACH),
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await;
    let Some(baseline) = baseline else {
        return Err("no baseline agreement".to_string());
    };
    eprintln!(
        "baseline: p1={} p2={} p3={} global={}",
        baseline.p1, baseline.p2, baseline.p3, baseline.global
    );
    soft(
        &mut run.checks,
        "baseline-agreement",
        baseline.global
            == baseline
                .p1
                .saturating_add(baseline.p2)
                .saturating_add(baseline.p3),
        format!("exact agreement after drive: {baseline:?}"),
    );
    run.baseline = Some(baseline);
    Ok(())
}

async fn phase_rejoin_3(run: &mut Run<'_>) -> Result<(), String> {
    kill_opt(&mut run.guard3);
    tokio::time::sleep(Duration::from_secs(20)).await;
    let after_leave = tick_lines(&run.log1).saturating_add(tick_lines(&run.log2));
    ensure(
        tick_lines(&run.log1) > 0 && tick_lines(&run.log2) > 0,
        "room stalled after app3 left".to_string(),
    )?;
    soft(
        &mut run.checks,
        "leave-survives",
        true,
        format!("room kept ticking after app3 left, ticks={after_leave}"),
    );
    let leave_logs = [run.log1.clone(), run.log2.clone()];
    assert_all_owners(
        &mut run.checks,
        "leave-owners",
        &leave_logs,
        2,
        LAG_AGREE_TIMEOUT_SECS,
    )
    .await;
    let before_rejoin = tick_lines(&run.log2);
    let Some((guard, log)) = spawn_puller(&PullerRequest {
        bin: run.bin,
        dir: run.persist_dir,
        namespace: run.namespace,
        tag: 3,
        contract_params: run.contract_params,
        transport: run.transport,
        mdns: run.mdns,
        room: &run.room,
        log_name: "instance-3-rejoin.log",
        tile: (960, 1080, 960, 0),
    })
    .await
    else {
        return Err("spawn puller 3 rejoin".to_string());
    };
    run.guard3 = Some(guard);
    run.log3 = log;
    let rejoin_pos = file_len(&run.log3);
    let Some(clicked_at) = ui_join_room(GAME_TITLES[2], &run.room, &run.log3, brp_port(3)).await
    else {
        return Err("instance 3 never rejoined room via menu".to_string());
    };
    assert_ready(
        &mut run.checks,
        "rejoin-ready-3",
        &run.log3,
        &run.room,
        rejoin_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    let rejoining = run.log3.clone();
    let survivor = run.log2.clone();
    let restored = await_rejoin(run, "rejoin-3", &rejoining, &survivor).await?;
    let baseline = run.baseline.ok_or_else(|| "baseline missing".to_string())?;
    ensure(
        restored.p1 >= baseline.p1 && restored.p2 >= baseline.p2 && restored.p3 >= baseline.p3,
        format!("app3 lost distributed state: baseline={baseline:?} restored={restored:?}"),
    )?;
    soft(
        &mut run.checks,
        "rejoin-restores",
        true,
        format!("no state lost on app3 rejoin: baseline={baseline:?} restored={restored:?}"),
    );
    ensure(
        tick_lines(&run.log2) > before_rejoin,
        "room stalled while app3 rejoined".to_string(),
    )?;
    soft(
        &mut run.checks,
        "rejoin-live",
        true,
        format!(
            "room kept ticking during app3 rejoin, ticks={}",
            tick_lines(&run.log2)
        ),
    );
    eprintln!("app3 rejoined with state intact");
    run.restored = Some(restored);
    Ok(())
}

async fn phase_rejoin_creator(run: &mut Run<'_>) -> Result<(), String> {
    kill_opt(&mut run.guard1);
    let creator_gone_ticks = tick_lines(&run.log2);
    tokio::time::sleep(Duration::from_secs(25)).await;
    ensure(
        tick_lines(&run.log2) > creator_gone_ticks,
        "room stalled after creator left: creator is not special, the room must survive"
            .to_string(),
    )?;
    soft(
        &mut run.checks,
        "creator-leaves-survives",
        true,
        format!(
            "room survived creator leave, ticks {} -> {}",
            creator_gone_ticks,
            tick_lines(&run.log2)
        ),
    );
    let Some((guard, log)) = spawn_puller(&PullerRequest {
        bin: run.bin,
        dir: run.persist_dir,
        namespace: run.namespace,
        tag: 1,
        contract_params: run.contract_params,
        transport: run.transport,
        mdns: run.mdns,
        room: &run.room,
        log_name: "instance-1-rejoin.log",
        tile: (960, 540, 0, 0),
    })
    .await
    else {
        return Err("spawn puller 1 rejoin".to_string());
    };
    run.guard1 = Some(guard);
    run.log1 = log;
    let rejoin_pos = file_len(&run.log1);
    let Some(clicked_at) = ui_join_room(GAME_TITLES[0], &run.room, &run.log1, brp_port(1)).await
    else {
        return Err("creator never rejoined room via menu".to_string());
    };
    assert_ready(
        &mut run.checks,
        "rejoin-ready-1",
        &run.log1,
        &run.room,
        rejoin_pos,
        clicked_at,
        READY_SECS,
    )
    .await;
    let rejoining = run.log1.clone();
    let survivor = run.log2.clone();
    let final_state = await_rejoin(run, "rejoin-1", &rejoining, &survivor).await?;
    let restored = run.restored.ok_or_else(|| "restored missing".to_string())?;
    ensure(
        final_state.p1 >= restored.p1
            && final_state.p2 >= restored.p2
            && final_state.p3 >= restored.p3,
        format!("creator rejoin broke the room: restored={restored:?} final={final_state:?}"),
    )?;
    eprintln!(
        "creator rejoined as ordinary peer: p1={} p2={} p3={} global={}",
        final_state.p1, final_state.p2, final_state.p3, final_state.global
    );
    soft(
        &mut run.checks,
        "creator-rejoin-agreement",
        final_state.p1 >= restored.p1
            && final_state.p2 >= restored.p2
            && final_state.p3 >= restored.p3,
        format!("creator back as ordinary peer: restored={restored:?} final={final_state:?}"),
    );
    Ok(())
}

async fn phase_leave_load(run: &mut Run<'_>) -> Result<(), String> {
    let Some((guard, log)) = spawn_puller(&PullerRequest {
        bin: run.bin,
        dir: run.persist_dir,
        namespace: run.namespace,
        tag: 4,
        contract_params: run.contract_params,
        transport: run.transport,
        mdns: run.mdns,
        room: &run.room,
        log_name: "instance-4.log",
        tile: (960, 540, 960, 540),
    })
    .await
    else {
        return Err("spawn puller 4".to_string());
    };
    run.guard4 = Some(guard);
    run.log4 = log;
    soft(
        &mut run.checks,
        "spawn-4-node",
        true,
        "instance 4 freenet node started".to_string(),
    );
    let join4_pos = file_len(&run.log4);
    ensure(
        ui_join_room("clicker-4", &run.room, &run.log4, brp_port(4))
            .await
            .is_some(),
        "instance 4 never joined room via menu".to_string(),
    )?;
    soft(
        &mut run.checks,
        "ui-join-4",
        true,
        format!("instance 4 joined {} via menu button", run.room),
    );
    let left4 = ui_leave_room("clicker-4", &run.log4, join4_pos).await;
    soft(
        &mut run.checks,
        "leave-during-loading-4",
        left4,
        "instance 4 left via the leave button right after joining".to_string(),
    );
    ensure(
        left4,
        "instance 4 never left via the leave button".to_string(),
    )?;
    let probe_ticks = tick_lines(&run.log2);
    tokio::time::sleep(Duration::from_secs(10)).await;
    let survived = tick_lines(&run.log2) > probe_ticks;
    soft(
        &mut run.checks,
        "leave-survives-4",
        survived,
        format!(
            "room kept ticking after instance 4 left, ticks {} -> {}",
            probe_ticks,
            tick_lines(&run.log2)
        ),
    );
    ensure(survived, "room stalled after instance 4 left".to_string())?;
    Ok(())
}

async fn phase_finish(
    run: &mut Run<'_>,
    test_log: &TestLog,
    recording: Option<std::process::Child>,
    raw_path: &std::path::Path,
) -> Result<(), String> {
    let clip_path = run.persist_dir.join("clip.mp4");
    let clip = recording
        .and_then(|child| finish_record(child, raw_path))
        .and_then(|raw| speed_clip_x4(&raw, &clip_path));
    kill_opt(&mut run.guard1);
    kill_opt(&mut run.guard2);
    kill_opt(&mut run.guard3);
    kill_opt(&mut run.guard4);
    let Some(clip) = clip else {
        test_log.line(&format!("clip missing at {}", clip_path.display()));
        return ensure(
            raw_path.exists() && clip_path.exists(),
            format!("clip missing at {}", clip_path.display()),
        );
    };
    let check_lines: Vec<String> = run
        .checks
        .iter()
        .map(|c| format!("{} {} {}", check_emoji(c.ok), c.name, c.detail))
        .collect();
    let all_ok = run.checks.iter().all(|c| c.ok);
    ensure(
        all_ok,
        format!("subtest failures:\n{}", check_lines.join("\n")),
    )?;
    let caption = format!(
        "clicker rooms-rejoin room={} transport={} mdns={} · {} ({} / {})\n{}\nlogs: {}",
        run.room,
        run.transport,
        run.mdns,
        check_emoji(all_ok),
        run.checks.iter().filter(|c| c.ok).count(),
        run.checks.len(),
        check_lines.join("\n"),
        run.persist_dir.display()
    );
    test_log.line(&format!("sending clip {}", clip_path.display()));
    if let Err(err) =
        tokio::task::spawn_blocking(move || send_video_best_effort(&clip, &caption)).await
    {
        test_log.line(&format!("telegram send join failed: {err}"));
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs X + 3 clicker windows + public Freenet mainnet; run with --ignored --nocapture"]
#[telegram_bot::telegram_notify]
async fn rooms_rejoin() {
    let test_log = TestLog::open("rooms_rejoin");
    test_log.line("test started");
    let bin = build_binary_for_rejoin();
    let persist_dir = prepare_persist_dir();
    let (namespace, contract_params, transport, mdns) = env_params();
    let raw_path = persist_dir.join("raw.mp4");
    let recording = start_record(RECORD_SECS, &raw_path);
    let mut run = Run {
        bin: &bin,
        persist_dir: &persist_dir,
        namespace: &namespace,
        contract_params: &contract_params,
        transport: &transport,
        mdns,
        checks: Vec::new(),
        room: String::new(),
        log1: persist_dir.join("instance-1.log"),
        log2: persist_dir.join("instance-2.log"),
        log3: persist_dir.join("instance-3.log"),
        log4: persist_dir.join("instance-4.log"),
        guard1: None,
        guard2: None,
        guard3: None,
        guard4: None,
        baseline: None,
        restored: None,
    };
    phase_creator(&mut run).await.expect("phase creator");
    phase_join_2(&mut run).await.expect("phase join 2");
    phase_join_3(&mut run).await.expect("phase join 3");
    phase_converge_drive(&mut run)
        .await
        .expect("phase converge");
    phase_rejoin_3(&mut run).await.expect("phase rejoin 3");
    phase_rejoin_creator(&mut run)
        .await
        .expect("phase rejoin creator");
    phase_leave_load(&mut run)
        .await
        .expect("phase leave during loading");
    phase_finish(&mut run, &test_log, recording, &raw_path)
        .await
        .expect("phase finish");
}
