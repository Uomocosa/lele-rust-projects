use std::time::{Duration, Instant};

use clicker_lib::testing::{
    TerminalGuard, build_game, cleanup_stale, drive_random, finish_record, poke, require_xterm,
    spawn_xterm, start_record, tile_three, wakeup_screen,
};
use telegram_bot::{load_creds, send_video_file};

const TIMEOUT_SECS: u64 = 300;
const CLIP_SECS: u64 = 25;
const GAME_TITLES: [&str; 3] = ["clicker-1", "clicker-2", "clicker-3"];
const OWN_IDS: [u64; 3] = [1, 2, 3];
const CLICKS_EACH: u32 = 15;
const FRESH_MS: i64 = 5000;
const TRAVEL_PX: f64 = 100.0;

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

fn parse_number_after(stripped: &str, needle: &str) -> Option<f64> {
    let idx = stripped.find(needle)?;
    let start = idx.checked_add(needle.len())?;
    let rest = stripped.get(start..)?;
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest.get(..end)?.parse::<f64>().ok()
}

fn parse_hues(path: &std::path::Path) -> Vec<(u64, f64)> {
    let mut hues = Vec::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return hues;
    };
    for line in content.lines() {
        let stripped = strip_ansi(line);
        if stripped.contains("cursor resolved") {
            let (Some(player), Some(hue)) = (
                parse_number_after(&stripped, " player=").map(|v| v as u64),
                parse_number_after(&stripped, " hue="),
            ) else {
                continue;
            };
            hues.push((player, hue));
        } else if stripped.contains("cursor color player=") && !stripped.contains("player=?") {
            let (Some(player), Some(hue)) = (
                parse_number_after(&stripped, " player=").map(|v| v as u64),
                parse_number_after(&stripped, " hue="),
            ) else {
                continue;
            };
            hues.push((player, hue));
        }
    }
    hues
}

fn expected_hue(player: u64) -> f64 {
    let hue = clicker_lib::clicker::hue_for(freenet_libp2p_bevy_plugin::net_id::NetworkId(player));
    format!("{hue:.1}").parse::<f64>().unwrap_or(f64::NAN)
}

fn color_report(terms: &[TerminalGuard]) -> SoftCheck {
    let mut ok = true;
    let mut parts = Vec::new();
    let mut agreed: Vec<(u64, f64)> = Vec::new();
    for player in OWN_IDS {
        let expected = expected_hue(player);
        let mut first: Option<f64> = None;
        let mut player_ok = true;
        let mut detail = String::new();
        for (i, guard) in terms.iter().enumerate() {
            let hues: Vec<f64> = parse_hues(&guard.log)
                .into_iter()
                .filter(|(p, _)| *p == player)
                .map(|(_, h)| h)
                .collect();
            if hues.is_empty() {
                player_ok = false;
                detail = format!("inst{} missing", i.saturating_add(1));
                break;
            }
            if hues.iter().any(|h| h.to_bits() != hues[0].to_bits()) {
                player_ok = false;
                detail = format!("inst{} drift", i.saturating_add(1));
                break;
            }
            if hues[0].to_bits() != expected.to_bits() {
                player_ok = false;
                detail = format!("inst{} wrong hue", i.saturating_add(1));
                break;
            }
            match first {
                None => first = Some(hues[0]),
                Some(f) if f.to_bits() != hues[0].to_bits() => {
                    player_ok = false;
                    detail = format!("inst{} disagree", i.saturating_add(1));
                    break;
                }
                _ => {}
            }
        }
        if player_ok {
            if let Some(f) = first {
                agreed.push((player, f));
                parts.push(format!("p{player}={f:.1}{}", check_emoji(true)));
            }
        } else {
            ok = false;
            parts.push(format!("p{player}=?{} {detail}", check_emoji(false)));
        }
    }
    if ok {
        for i in 0..agreed.len() {
            for j in (i.saturating_add(1))..agreed.len() {
                if agreed[i].1.to_bits() == agreed[j].1.to_bits() {
                    ok = false;
                    parts.push(format!(
                        "collision p{}=p{}{}",
                        agreed[i].0,
                        agreed[j].0,
                        check_emoji(false)
                    ));
                }
            }
        }
    }
    SoftCheck {
        name: "color-agree".to_string(),
        ok,
        detail: parts.join(" "),
    }
}

fn expected_color(player: u64) -> String {
    let base =
        clicker_lib::clicker::color_for(freenet_libp2p_bevy_plugin::net_id::NetworkId(player));
    format!("{base:?}")
}

fn parse_flash(path: &std::path::Path) -> Vec<(Option<u64>, String)> {
    let mut out = Vec::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return out;
    };
    for line in content.lines() {
        let stripped = strip_ansi(line);
        if !stripped.contains("cursor flash done") {
            continue;
        }
        let player = parse_number_after(&stripped, " player=").map(|v| v as u64);
        let Some(color_idx) = stripped.find(" color=") else {
            continue;
        };
        let Some(start) = color_idx.checked_add(7) else {
            continue;
        };
        let Some(color) = stripped.get(start..).map(str::trim_end).map(str::to_string) else {
            continue;
        };
        if player.is_none() && !stripped.contains("player=?") {
            continue;
        }
        out.push((player, color));
    }
    out
}

fn flash_report(terms: &[TerminalGuard]) -> SoftCheck {
    let mut ok = true;
    let mut parts = Vec::new();
    let mut agreed: Vec<(u64, String)> = Vec::new();
    for player in OWN_IDS {
        let expected = expected_color(player);
        let mut first: Option<String> = None;
        let mut player_ok = true;
        let mut detail = String::new();
        for (i, guard) in terms.iter().enumerate() {
            let colors: Vec<String> = parse_flash(&guard.log)
                .into_iter()
                .filter(|(p, _)| *p == Some(player))
                .map(|(_, c)| c)
                .collect();
            if colors.is_empty() {
                player_ok = false;
                detail = format!("inst{} missing", i.saturating_add(1));
                break;
            }
            if colors.iter().any(|c| *c != colors[0]) {
                player_ok = false;
                detail = format!("inst{} drift", i.saturating_add(1));
                break;
            }
            if colors[0] != expected {
                player_ok = false;
                detail = format!("inst{} wrong color", i.saturating_add(1));
                break;
            }
            if let Some(f) = first.as_ref() {
                if *f != colors[0] {
                    player_ok = false;
                    detail = format!("inst{} disagree", i.saturating_add(1));
                    break;
                }
            } else {
                first = Some(colors[0].clone());
            }
        }
        if player_ok {
            if let Some(f) = first {
                parts.push(format!("p{player}{}", check_emoji(true)));
                agreed.push((player, f));
            }
        } else {
            ok = false;
            parts.push(format!("p{player}=?{} {detail}", check_emoji(false)));
        }
    }
    if ok {
        for i in 0..agreed.len() {
            for j in (i.saturating_add(1))..agreed.len() {
                if agreed[i].1 == agreed[j].1 {
                    ok = false;
                    parts.push(format!(
                        "collision p{}=p{}{}",
                        agreed[i].0,
                        agreed[j].0,
                        check_emoji(false)
                    ));
                }
            }
        }
    }
    SoftCheck {
        name: "flash-hue-agree".to_string(),
        ok,
        detail: parts.join(" "),
    }
}

struct PosSample {
    player: u64,
    x: f64,
    y: f64,
    ts_ms: i64,
}

fn parse_pos(path: &std::path::Path) -> Vec<PosSample> {
    let mut samples = Vec::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return samples;
    };
    for line in content.lines() {
        let stripped = strip_ansi(line);
        if !stripped.contains("pos lobby=") {
            continue;
        }
        let (Some(player), Some(x), Some(y)) = (
            parse_number_after(&stripped, " player=").map(|v| v as u64),
            parse_number_after(&stripped, " x="),
            parse_number_after(&stripped, " y="),
        ) else {
            continue;
        };
        let ts_ms = stripped
            .split_whitespace()
            .next()
            .and_then(|token| chrono::DateTime::parse_from_rfc3339(token).ok())
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(-1);
        if ts_ms < 0 {
            continue;
        }
        samples.push(PosSample {
            player,
            x,
            y,
            ts_ms,
        });
    }
    samples
}

fn move_report(terms: &[TerminalGuard]) -> Vec<SoftCheck> {
    let mut fresh_ok = true;
    let mut travel_ok = true;
    let mut fresh_detail = "all fresh".to_string();
    let mut travel_detail = "all moved".to_string();
    for (i, guard) in terms.iter().enumerate() {
        let samples = parse_pos(&guard.log);
        for player in OWN_IDS {
            let mut series: Vec<&PosSample> =
                samples.iter().filter(|s| s.player == player).collect();
            series.sort_by_key(|s| s.ts_ms);
            let Some(last) = series.last() else {
                fresh_ok = false;
                travel_ok = false;
                fresh_detail = format!("inst{} p{player} no samples", i.saturating_add(1));
                travel_detail = fresh_detail.clone();
                continue;
            };
            if series.len() < 2 {
                fresh_ok = false;
                travel_ok = false;
                fresh_detail = format!("inst{} p{player} 1 sample", i.saturating_add(1));
                travel_detail = fresh_detail.clone();
                continue;
            }
            let mut worst_gap = 0_i64;
            let mut min_x = last.x;
            let mut max_x = last.x;
            let mut min_y = last.y;
            let mut max_y = last.y;
            let mut prev = series.first().map(|s| s.ts_ms).unwrap_or(0);
            for sample in &series {
                let gap = sample.ts_ms.checked_sub(prev).unwrap_or(i64::MAX);
                worst_gap = worst_gap.max(gap);
                min_x = min_x.min(sample.x);
                max_x = max_x.max(sample.x);
                min_y = min_y.min(sample.y);
                max_y = max_y.max(sample.y);
                prev = sample.ts_ms;
            }
            if worst_gap > FRESH_MS {
                fresh_ok = false;
                fresh_detail = format!("inst{} p{player} gap={worst_gap}ms", i.saturating_add(1));
            }
            let travel = (max_x - min_x).hypot(max_y - min_y);
            if travel < TRAVEL_PX {
                travel_ok = false;
                travel_detail = format!("inst{} p{player} disp={travel:.0}px", i.saturating_add(1));
            }
        }
    }
    vec![
        SoftCheck {
            name: "move-fresh-5s".to_string(),
            ok: fresh_ok,
            detail: fresh_detail,
        },
        SoftCheck {
            name: "cursor-travel-100px".to_string(),
            ok: travel_ok,
            detail: travel_detail,
        },
    ]
}

fn player_report(terms: &[TerminalGuard]) -> SoftCheck {
    let mut ok = true;
    let mut parts = Vec::new();
    for (i, guard) in terms.iter().enumerate() {
        let samples = parse_pos(&guard.log);
        let mut seen = Vec::new();
        for sample in &samples {
            if OWN_IDS.contains(&sample.player) && !seen.contains(&sample.player) {
                seen.push(sample.player);
            }
        }
        let mut missing = Vec::new();
        for player in OWN_IDS {
            if !seen.contains(&player) {
                missing.push(player);
            }
        }
        let good = missing.is_empty();
        ok &= good;
        if good {
            parts.push(format!(
                "inst{} 3/3{}",
                i.saturating_add(1),
                check_emoji(true)
            ));
        } else {
            parts.push(format!(
                "inst{} missing={missing:?}{}",
                i.saturating_add(1),
                check_emoji(false)
            ));
        }
    }
    SoftCheck {
        name: "logical-players".to_string(),
        ok,
        detail: parts.join("; "),
    }
}

fn log_contains(path: &std::path::Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|s| s.contains(needle))
}

fn room_resolved(path: &std::path::Path, room: &str) -> bool {
    log_contains(path, "room resolved") && log_contains(path, room)
}

// Auto-joiners resolve before Bevy installs its log subscriber, so the
// discovery-side marker never reaches their log; gate them on the post-App
// lobby instead, which carries the same unique room name.
fn room_joined(path: &std::path::Path, room: &str) -> bool {
    log_contains(path, &format!("lobby={room}"))
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

fn parse_last_count(path: &std::path::Path, owner: u64) -> Option<u64> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut last = None;
    for line in content.lines() {
        if !line.contains("tick lobby=") {
            continue;
        }
        let needle = format!("owner={owner} count=");
        let stripped = strip_ansi(line);
        let Some(idx) = stripped.find(&needle) else {
            continue;
        };
        let Some(start) = idx.checked_add(needle.len()) else {
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
        if let Ok(value) = num_str.parse::<u64>() {
            last = Some(value);
        }
    }
    last
}

fn parse_last_global(path: &std::path::Path) -> Option<u64> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut last = None;
    for line in content.lines() {
        let stripped = strip_ansi(line);
        let Some(idx) = stripped.find("global=") else {
            continue;
        };
        let Some(start) = idx.checked_add(7) else {
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
        if let Ok(value) = num_str.parse::<u64>() {
            last = Some(value);
        }
    }
    last
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

const LAG_SPREAD_BUDGET_MS: u128 = 2000;
const LAG_AGREE_TIMEOUT_SECS: u64 = 60;

async fn probe_lag(terms: &[TerminalGuard], drive_end: Instant) -> SoftCheck {
    let start = Instant::now();
    let mut history: Vec<(u128, Vec<Option<SyncState>>)> = Vec::new();
    let mut agreed: Option<(SyncState, u128)> = None;
    while start.elapsed().as_secs() < LAG_AGREE_TIMEOUT_SECS {
        let elapsed_ms = drive_end.elapsed().as_millis();
        let states: Vec<Option<SyncState>> =
            terms.iter().map(|g| parse_last_sync(&g.log)).collect();
        history.push((elapsed_ms, states.clone()));
        if states.len() == 3
            && let [Some(a), Some(b), Some(c)] = states.as_slice()
            && a == b
            && b == c
            && a.global == a.p1.saturating_add(a.p2).saturating_add(a.p3)
            && a.p1 >= CLICKS_EACH as u64
            && a.p2 >= CLICKS_EACH as u64
            && a.p3 >= CLICKS_EACH as u64
        {
            agreed = Some((*a, elapsed_ms));
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let Some((final_state, agree_ms)) = agreed else {
        return SoftCheck {
            name: "lag-agree".to_string(),
            ok: false,
            detail: "no agreement within 60s".to_string(),
        };
    };
    let mut first_seen = [None; 3];
    for (ms, states) in &history {
        for (i, state) in states.iter().enumerate() {
            if first_seen[i].is_none() && *state == Some(final_state) {
                first_seen[i] = Some(*ms);
            }
        }
    }
    let mut spread_ms = 0_u128;
    if first_seen.iter().all(|v| v.is_some()) {
        let min = first_seen.iter().filter_map(|v| *v).min().unwrap_or(0);
        let max = first_seen.iter().filter_map(|v| *v).max().unwrap_or(0);
        spread_ms = max.saturating_sub(min);
    }
    let ok = spread_ms <= LAG_SPREAD_BUDGET_MS;
    SoftCheck {
        name: "lag-spread-2s".to_string(),
        ok,
        detail: format!(
            "agree={agree_ms}ms spread={spread_ms}ms budget={}ms p1={} p2={} p3={} global={} {}",
            LAG_SPREAD_BUDGET_MS,
            final_state.p1,
            final_state.p2,
            final_state.p3,
            final_state.global,
            check_emoji(ok)
        ),
    }
}

fn resolved_count(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|content| content.matches("cursor resolved peer=").count())
        .unwrap_or_default()
}

fn parse_owners(path: &std::path::Path) -> Vec<u64> {
    let mut owners = Vec::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return owners;
    };
    for line in content.lines() {
        if !line.contains("tick lobby=") {
            continue;
        }
        let stripped = strip_ansi(line);
        let Some(tick_idx) = stripped.find("owner=") else {
            continue;
        };
        let Some(start) = tick_idx.checked_add(6) else {
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
        if let Ok(owner) = num_str.parse::<u64>()
            && !owners.contains(&owner)
        {
            owners.push(owner);
        }
    }
    owners
}

fn local_count(path: &std::path::Path, tag: u64) -> Option<u64> {
    parse_last_count(path, tag)
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

fn fmt_secs(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64())
}

const fn check_emoji(ok: bool) -> &'static str {
    if ok { "✅" } else { "❌" }
}

fn kill_terms(terms: &mut Vec<TerminalGuard>) {
    for guard in terms {
        if let Some(child) = guard.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn mesh_report(terms: &[TerminalGuard]) -> (bool, Vec<String>) {
    let mut ok = true;
    let mut lines = Vec::new();
    for (i, guard) in terms.iter().enumerate() {
        let owners = parse_owners(&guard.log);
        let tag = OWN_IDS.get(i).copied().unwrap_or_default();
        let local = local_count(&guard.log, tag).unwrap_or_default();
        let global = parse_last_global(&guard.log).unwrap_or_default();
        let mut min_remote = u64::MAX;
        for owner in &owners {
            let last = parse_last_count(&guard.log, *owner).unwrap_or_default();
            min_remote = min_remote.min(last);
        }
        let good = owners.len() == OWN_IDS.len() && local >= 15 && min_remote >= 15 && global >= 45;
        ok &= good;
        lines.push(format!(
            "inst{} owners={} local={local} min_remote={min_remote} global={global} {}",
            i.saturating_add(1),
            owners.len(),
            check_emoji(good)
        ));
    }
    (ok, lines)
}

fn spawn_tag(
    terms: &mut Vec<TerminalGuard>,
    bin: &std::path::Path,
    dir: &std::path::Path,
    tag: u64,
    contract_params: &str,
    lobby: Option<&str>,
    since_epoch: Option<u64>,
) -> bool {
    let log = dir.join(format!("instance-{tag}.log"));
    match spawn_xterm(
        bin,
        "blackboard-v1",
        lobby,
        tag == 1,
        tag,
        contract_params,
        since_epoch,
        &log,
    ) {
        Ok(guard) => {
            terms.push(guard);
            true
        }
        Err(e) => {
            eprintln!("spawn xterm {tag}: {e}");
            false
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "local-mainnet: needs X + 3 clicker windows + public Freenet mainnet; run with --ignored --nocapture"]
async fn local_mesh() {
    let total_start = Instant::now();
    wakeup_screen();
    assert!(require_xterm().is_ok(), "xterm/xdotool/wmctrl missing");
    cleanup_stale();
    let build_start = Instant::now();
    let mut bin = std::path::PathBuf::new();
    match build_game() {
        Ok(path) => bin = path,
        Err(e) => eprintln!("build clicker release: {e}"),
    }
    let build_elapsed = build_start.elapsed();
    assert!(bin.exists(), "binary not found: {}", bin.display());

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let persist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("clicker-{timestamp}"));
    assert!(
        std::fs::create_dir_all(&persist_dir).is_ok(),
        "create {}",
        persist_dir.display()
    );

    let mut terms = Vec::new();
    let test_start_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let room = format!("room-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let contract_params = std::env::var("CLICKER_CONTRACT_PARAMS")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_default();
    assert!(
        spawn_tag(
            &mut terms,
            &bin,
            &persist_dir,
            1,
            &contract_params,
            Some(&room),
            None
        ),
        "spawn 1"
    );
    let log0 = persist_dir.join("instance-1.log");
    assert!(
        wait_until(180, || log_contains(&log0, "discovery: roster connected")).await,
        "instance 1 never joined roster"
    );
    assert!(
        wait_until(60, || room_resolved(&log0, &room)).await,
        "instance 1 never resolved room"
    );
    assert!(
        spawn_tag(
            &mut terms,
            &bin,
            &persist_dir,
            2,
            &contract_params,
            None,
            Some(test_start_epoch)
        ),
        "spawn 2"
    );
    let log1 = persist_dir.join("instance-2.log");
    assert!(
        wait_until(180, || log_contains(&log1, "discovery: roster connected")).await,
        "instance 2 never joined roster"
    );
    assert!(
        wait_until(180, || room_joined(&log1, &room)).await,
        "instance 2 never auto-joined room"
    );
    assert!(
        spawn_tag(
            &mut terms,
            &bin,
            &persist_dir,
            3,
            &contract_params,
            None,
            Some(test_start_epoch)
        ),
        "spawn 3"
    );
    let log2 = persist_dir.join("instance-3.log");
    assert!(
        wait_until(180, || room_joined(&log2, &room)).await,
        "instance 3 never auto-joined room"
    );
    assert!(tile_three(GAME_TITLES).is_ok(), "tile game windows");

    let room_marker = format!("lobby={room}");
    let converged = wait_until(TIMEOUT_SECS, || {
        terms
            .iter()
            .all(|g| log_contains(&g.log, "connected, running indefinitely"))
            && terms.iter().all(|g| log_contains(&g.log, "tick lobby="))
            && terms.iter().all(|g| log_contains(&g.log, &room_marker))
            && terms
                .iter()
                .all(|g| log_contains(&g.log, "accounting for remote owner="))
            && terms.iter().all(|g| resolved_count(&g.log) >= 2)
    })
    .await;

    for title in GAME_TITLES {
        eprintln!("drive_random {title} x{CLICKS_EACH}");
        let drive_result =
            tokio::task::spawn_blocking(move || drive_random(title, CLICKS_EACH)).await;
        match drive_result {
            Ok(Ok(())) => eprintln!("drive_random {title} done"),
            Ok(Err(err)) => eprintln!("drive_random {title} failed: {err}"),
            Err(err) => eprintln!("drive_random {title} join failed: {err}"),
        }
    }
    for _ in 0..3 {
        let mut short: Vec<(&str, u64)> = Vec::new();
        for (tag, title) in OWN_IDS.iter().zip(GAME_TITLES.iter()) {
            let log = persist_dir.join(format!("instance-{tag}.log"));
            let local = local_count(&log, *tag).unwrap_or_default();
            if local < CLICKS_EACH as u64 {
                short.push((title, (CLICKS_EACH as u64).saturating_sub(local)));
            }
        }
        if short.is_empty() {
            break;
        }
        for (title, missing) in short {
            eprintln!("top-up {title} x{missing}");
            let count = u32::try_from(missing).unwrap_or(1);
            let drive_result =
                tokio::task::spawn_blocking(move || drive_random(title, count)).await;
            match drive_result {
                Ok(Ok(())) => eprintln!("top-up {title} done"),
                Ok(Err(err)) => eprintln!("top-up {title} failed: {err}"),
                Err(err) => eprintln!("top-up {title} join failed: {err}"),
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    let drive_end = Instant::now();
    eprintln!("drive done, waiting for settle");

    let auto_ok = wait_until(180, || {
        OWN_IDS.iter().enumerate().all(|(i, tag)| {
            terms
                .get(i)
                .is_some_and(|g| local_count(&g.log, *tag).is_some_and(|c| c >= 15))
        })
    })
    .await;
    tokio::time::sleep(Duration::from_secs(15)).await;
    eprintln!("probe_lag from settle end");
    let lag_check = probe_lag(&terms, drive_end).await;
    eprintln!("probe_lag done: {} {}", lag_check.name, lag_check.detail);

    let (mesh_ok, mesh_lines) = mesh_report(&terms);
    let mesh_ok = auto_ok && mesh_ok;

    let mut checks = Vec::new();
    soft(&mut checks, "mesh-counts", mesh_ok, mesh_lines.join("; "));
    checks.push(lag_check);
    checks.push(color_report(&terms));
    checks.push(flash_report(&terms));
    checks.push(player_report(&terms));
    checks.extend(move_report(&terms));

    let recording_start = Instant::now();
    let clip_path = persist_dir.join("clip.mp4");
    let record_path = clip_path.clone();
    let video = tokio::task::spawn_blocking(move || {
        start_record(CLIP_SECS, &record_path).and_then(|child| {
            std::thread::sleep(Duration::from_secs(CLIP_SECS));
            finish_record(child, &record_path)
        })
    })
    .await
    .unwrap_or(None);
    let recording_elapsed = recording_start.elapsed();
    kill_terms(&mut terms);
    let total_elapsed = total_start.elapsed();

    let Some(clip) = video else {
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
    let caption = format!(
        "clicker local-mesh room={room} · {} converged={converged} {} all_ok={all_ok}\n{}\n{}\nlogs: {} · build {} s · recording {} s · total {} s",
        check_emoji(converged),
        check_emoji(all_ok),
        mesh_lines.join("\n"),
        check_lines.join("\n"),
        persist_dir.display(),
        fmt_secs(build_elapsed),
        fmt_secs(recording_elapsed),
        fmt_secs(total_elapsed)
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
    assert!(converged, "no converge: {}", persist_dir.display());
    for c in &checks {
        assert!(c.ok, "soft check {} failed: {}", c.name, c.detail);
    }
}
