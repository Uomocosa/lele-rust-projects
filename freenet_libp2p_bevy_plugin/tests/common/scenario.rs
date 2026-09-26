use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use telegram_bot::send_video_best_effort;

use super::{
    TerminalGuard, build_game, cleanup_stale, finish_record, last_tick, log_contains, poke,
    read_head, require_x11, speed_clip, start_record, tile, wakeup_screen,
};

pub const DISCOVERY_BUDGET_SECS: u64 = 120;
pub const JOIN_BUDGET_SECS: u64 = 180;

const SPAWN_TIMEOUT_SECS: u64 = 180;
const RECORD_SECS: u64 = 600;
const POLL_MS: u64 = 500;
const SPAWN_GAP_MS: u64 = 300;

pub struct Scenario {
    pub name: String,
    pub peers: usize,
    pub join: bool,
    pub discovery_budget_secs: u64,
    pub join_budget_secs: u64,
}

impl Scenario {
    #[must_use]
    pub fn discover_only(peers: usize) -> Self {
        Self {
            name: "lobby_room_discovery".to_string(),
            peers,
            join: false,
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }

    #[must_use]
    pub fn join(peers: usize) -> Self {
        Self {
            name: "lobby_room_join".to_string(),
            peers,
            join: true,
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }
}

pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

pub struct Report {
    pub checks: Vec<Check>,
    pub caption: String,
    pub logs_dir: PathBuf,
}

impl Report {
    pub fn assert_all(&self) {
        for check in &self.checks {
            assert!(
                check.ok,
                "check `{}` failed: {} (logs: {})",
                check.name,
                check.detail,
                self.logs_dir.display()
            );
        }
    }
}

struct Env {
    bin: PathBuf,
    dir: PathBuf,
    room: String,
    token: String,
    transport: String,
    raw: PathBuf,
    clip: PathBuf,
    build_secs: f64,
    recording: Option<Child>,
}

struct Peers {
    windows: Vec<TerminalGuard>,
    logs: Vec<PathBuf>,
    created: bool,
    spawn_ok: bool,
    t_create: Instant,
    t_join: Instant,
}

pub async fn run_scenario(scenario: Scenario) -> Report {
    let mut env = match prepare(&scenario) {
        Ok(env) => env,
        Err(detail) => return early_report(&scenario, detail),
    };
    let mut peers = spawn(&scenario, &env);
    let checks = evaluate(&scenario, &env, &peers).await;
    let clip = stop_recording(env.recording.take(), &env.raw, &env.clip);
    let windows = std::mem::take(&mut peers.windows);
    release(windows);
    let caption = build_caption(&scenario, &env, &peers, &checks);
    eprintln!("{caption}");
    send_clip(clip.as_deref(), &caption).await;
    Report {
        checks,
        caption,
        logs_dir: env.dir,
    }
}

fn prepare(scenario: &Scenario) -> Result<Env, String> {
    wakeup_screen();
    require_x11()?;
    cleanup_stale();
    let build_start = Instant::now();
    let bin = build_game()?;
    let build_secs = build_start.elapsed().as_secs_f64();
    let stamp = epoch_secs();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("{}-{stamp}", scenario.name));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let raw = dir.join("raw.mp4");
    let clip = dir.join("clip.mp4");
    let recording = start_record(RECORD_SECS, &raw);
    Ok(Env {
        bin,
        dir,
        room: format!("room-{stamp}"),
        token: format!("lobby-e2e/{stamp}"),
        transport: std::env::var("LOBBY_TRANSPORT").unwrap_or_else(|_| "both".to_string()),
        raw,
        clip,
        build_secs,
        recording,
    })
}

fn spawn(scenario: &Scenario, env: &Env) -> Peers {
    let mut windows = Vec::new();
    let mut logs = Vec::new();
    let mut spawn_ok = true;
    let create = format!("create:{}", env.room);
    if let Err(e) = push_peer(env, "p1", Some(&create), &mut windows, &mut logs) {
        eprintln!("{e}");
        spawn_ok = false;
    }
    let created = logs.first().is_some_and(|log| {
        wait_log(
            log,
            &format!("lobby created room={}", env.room),
            SPAWN_TIMEOUT_SECS,
        )
    });
    let t_create = Instant::now();
    for index in 2..=scenario.peers {
        let username = format!("p{index}");
        let action = scenario.join.then(|| format!("join:{}", env.room));
        if let Err(e) = push_peer(env, &username, action.as_deref(), &mut windows, &mut logs) {
            eprintln!("{e}");
            spawn_ok = false;
        }
        std::thread::sleep(Duration::from_millis(SPAWN_GAP_MS));
    }
    let t_join = Instant::now();
    if let Err(e) = tile(&windows) {
        eprintln!("tile: {e}");
    }
    Peers {
        windows,
        logs,
        created,
        spawn_ok,
        t_create,
        t_join,
    }
}

fn push_peer(
    env: &Env,
    username: &str,
    action: Option<&str>,
    windows: &mut Vec<TerminalGuard>,
    logs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let log = spawn_peer(
        &env.bin,
        &env.dir,
        username,
        action,
        &env.token,
        &env.transport,
        windows,
    )?;
    logs.push(log);
    Ok(())
}

async fn evaluate(scenario: &Scenario, env: &Env, peers: &Peers) -> Vec<Check> {
    let seen = wait_until(Duration::from_secs(scenario.discovery_budget_secs), || {
        all_see_room(&peers.logs, &env.room)
    })
    .await;
    let discovery_ms = peers.t_create.elapsed().as_millis();
    let discovery_ok = peers.created
        && peers.spawn_ok
        && seen
        && peers.t_create.elapsed().as_secs() <= scenario.discovery_budget_secs;
    let (join_ok, join_ms, join_detail) = evaluate_join(scenario, env, peers).await;
    let spawn_check = Check {
        name: "spawn".to_string(),
        ok: peers.spawn_ok && peers.logs.len() == scenario.peers,
        detail: format!("{}/{} instances", peers.logs.len(), scenario.peers),
    };
    let discovery_check = Check {
        name: "discovery".to_string(),
        ok: discovery_ok,
        detail: format!(
            "{discovery_ms}ms budget={}s created={} seen={seen}",
            scenario.discovery_budget_secs, peers.created
        ),
    };
    let join_check = Check {
        name: "join".to_string(),
        ok: join_ok,
        detail: format!(
            "{join_ms}ms budget={}s {join_detail}",
            scenario.join_budget_secs
        ),
    };
    vec![spawn_check, discovery_check, join_check]
}

async fn evaluate_join(scenario: &Scenario, env: &Env, peers: &Peers) -> (bool, u128, String) {
    if !scenario.join {
        return (true, 0, "n/a".to_string());
    }
    let mesh = wait_until(Duration::from_secs(scenario.join_budget_secs), || {
        all_in_room(&peers.logs, &env.room, scenario.peers)
    })
    .await;
    let budget_ok = peers.t_join.elapsed().as_secs() <= scenario.join_budget_secs;
    let detail = if mesh {
        "full mesh".to_string()
    } else {
        format!(
            "mesh not reached: {}",
            mesh_snapshot(&peers.logs, scenario.peers)
        )
    };
    (
        mesh && budget_ok,
        peers.t_join.elapsed().as_millis(),
        detail,
    )
}

fn stop_recording(recording: Option<Child>, raw: &Path, clip: &Path) -> Option<PathBuf> {
    recording
        .and_then(|child| finish_record(child, raw))
        .and_then(|raw| speed_clip(&raw, clip))
}

fn release(windows: Vec<TerminalGuard>) {
    if std::env::var("LOBBY_KEEP").is_err() {
        drop(windows);
    } else {
        std::mem::forget(windows);
    }
}

fn build_caption(scenario: &Scenario, env: &Env, peers: &Peers, checks: &[Check]) -> String {
    let mut lines = vec![
        freenet_status(env, &peers.logs),
        format!(
            "{} room={} transport={} peers={} · build {:.1}s",
            scenario.name, env.room, env.transport, scenario.peers, env.build_secs
        ),
        format!("created={} spawn_ok={}", peers.created, peers.spawn_ok),
    ];
    for check in checks {
        lines.push(format!(
            "{} {} {}",
            if check.ok { "✅" } else { "❌" },
            check.name,
            check.detail
        ));
    }
    lines.push(format!("logs: {}", env.dir.display()));
    lines.push(
        "rule: peers learn each other ONLY via the freenet board (no mdns/kad/bootstrap)"
            .to_string(),
    );
    lines.join("\n")
}

// needed helper: informational freenet network status line (no pass/fail)
fn freenet_status(env: &Env, logs: &[PathBuf]) -> String {
    let text = logs
        .first()
        .map_or_else(String::new, |log| read_head(log, 262_144));
    let network = if text.contains("Replacing local gateways with gateways from remote index") {
        "mainnet"
    } else {
        "local/isolated"
    };
    format!(
        "running on: {network} gateway={} mdns=off transport={} token={}",
        gateway_addr(&text),
        env.transport,
        env.token
    )
}

// needed helper: extracts the first gateway ip:port advertised in the log head
fn gateway_addr(text: &str) -> String {
    const MARKER: &str = "gateway gateway=";
    text.lines()
        .find_map(|line| {
            let index = line.find(MARKER)?;
            let rest = line.get(index.checked_add(MARKER.len())?..)?;
            let entry = rest.split_whitespace().next()?;
            entry.split('@').next_back().map(str::to_string)
        })
        .unwrap_or_else(|| "none".to_string())
}

async fn send_clip(clip: Option<&Path>, caption: &str) {
    let Some(clip) = clip else {
        return;
    };
    let clip = clip.to_path_buf();
    let caption = caption.to_string();
    let _ = tokio::task::spawn_blocking(move || send_video_best_effort(&clip, &caption)).await;
}

fn early_report(scenario: &Scenario, detail: String) -> Report {
    Report {
        checks: vec![Check {
            name: "setup".to_string(),
            ok: false,
            detail,
        }],
        caption: format!("{} setup failed", scenario.name),
        logs_dir: PathBuf::new(),
    }
}

// needed helper: spawns one app instance with its action/token args
fn spawn_peer(
    bin: &Path,
    dir: &Path,
    username: &str,
    action: Option<&str>,
    token: &str,
    transport: &str,
    windows: &mut Vec<TerminalGuard>,
) -> Result<PathBuf, String> {
    let log = dir.join(format!("instance-{username}.log"));
    let mut args = vec![
        "--username".to_string(),
        username.to_string(),
        "--token".to_string(),
        token.to_string(),
        "--transport".to_string(),
        transport.to_string(),
    ];
    if let Some(action) = action {
        args.push("--action".to_string());
        args.push(action.to_string());
    }
    let guard = super::spawn_app::spawn_app(bin, &args, username, &log)?;
    windows.push(guard);
    Ok(log)
}

// needed helper: every peer's latest tick lists the room in the catalogue
fn all_see_room(logs: &[PathBuf], room: &str) -> bool {
    logs.iter()
        .all(|log| last_tick(log).is_some_and(|tick| tick.catalogue.iter().any(|r| r == room)))
}

// needed helper: every peer is in the room with a full mesh
fn all_in_room(logs: &[PathBuf], room: &str, peers: usize) -> bool {
    logs.iter().all(|log| {
        last_tick(log).is_some_and(|tick| {
            tick.room.as_deref() == Some(room) && tick.connected == peers.saturating_sub(1)
        })
    })
}

// needed helper: current per-peer tick summary for failure details
fn mesh_snapshot(logs: &[PathBuf], peers: usize) -> String {
    let want = peers.saturating_sub(1);
    logs.iter()
        .enumerate()
        .map(|(index, log)| {
            let tick = last_tick(log);
            let room = tick
                .as_ref()
                .and_then(|t| t.room.clone())
                .unwrap_or_else(|| "none".to_string());
            let connected = tick.as_ref().map_or(0, |t| t.connected);
            format!(
                "p{}:room={room},connected={connected}/{want}",
                index.saturating_add(1)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// needed helper: polls one log until it contains a marker or the timeout elapses
fn wait_log(log: &Path, needle: &str, timeout_secs: u64) -> bool {
    let start = Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if log_contains(log, needle) {
            return true;
        }
        poke();
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    log_contains(log, needle)
}

// needed helper: async polling predicate with screen poke
async fn wait_until(timeout: Duration, mut predicate: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if predicate() {
            return true;
        }
        poke();
        tokio::time::sleep(Duration::from_millis(POLL_MS)).await;
    }
    predicate()
}

// needed helper: unix seconds for unique run/room names
fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
