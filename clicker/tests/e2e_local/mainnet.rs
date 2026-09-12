use std::time::{Duration, Instant};

use clicker_lib::testing::{
    TerminalGuard, build_game, drive_random, finish_record, load_creds, poke, require_xterm,
    send_video_file, spawn_xterm, start_record, tile_three, wakeup_screen,
};

const TIMEOUT_SECS: u64 = 300;
const CLIP_SECS: u64 = 25;
const LOBBY: &str = "alpha";
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

fn color_report(terms: &[TerminalGuard]) -> SoftCheck {
    let mut ok = true;
    let mut parts = Vec::new();
    for player in OWN_IDS {
        let mut seen = Vec::new();
        for guard in terms {
            for (p, hue) in parse_hues(&guard.log) {
                if p == player {
                    seen.push(hue);
                    break;
                }
            }
        }
        let agree = seen.len() == terms.len()
            && seen
                .iter()
                .all(|h| (*h - seen.first().copied().unwrap_or_default()).abs() <= 0.5);
        ok &= agree;
        let shown = seen
            .first()
            .map(|h| format!("{h:.1}"))
            .unwrap_or_else(|| "?".to_string());
        parts.push(format!("p{player}={shown}{}", check_emoji(agree)));
    }
    SoftCheck {
        name: "color-agree".to_string(),
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

fn parse_ready_addr(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if !line.contains("ready peer_id=") || !line.contains("addrs=") {
            continue;
        }
        let stripped = strip_ansi(line);
        for part in stripped.split('"') {
            if part.contains("/tcp/") {
                return Some(part.to_string());
            }
        }
    }
    None
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
    dial: &[String],
) -> bool {
    let log = dir.join(format!("instance-{tag}.log"));
    match spawn_xterm(bin, "blackboard-v1", LOBBY, tag == 1, tag, dial, &log) {
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
#[ignore = "local mesh: needs X + 3 clicker windows; run with --ignored --nocapture"]
async fn local_mesh() {
    let total_start = Instant::now();
    wakeup_screen();
    assert!(require_xterm().is_ok(), "xterm/xdotool/wmctrl missing");
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
    assert!(spawn_tag(&mut terms, &bin, &persist_dir, 1, &[]), "spawn 1");
    let log0 = persist_dir.join("instance-1.log");
    let ready_ok = wait_until(180, || parse_ready_addr(&log0).is_some()).await;
    let dial_addr1 = parse_ready_addr(&log0).unwrap_or_default();
    assert!(ready_ok && !dial_addr1.is_empty(), "instance 1 never ready");
    assert!(
        spawn_tag(
            &mut terms,
            &bin,
            &persist_dir,
            2,
            std::slice::from_ref(&dial_addr1)
        ),
        "spawn 2"
    );
    let log1 = persist_dir.join("instance-2.log");
    let second_ready = wait_until(180, || parse_ready_addr(&log1).is_some()).await;
    let dial_addr2 = parse_ready_addr(&log1).unwrap_or_default();
    assert!(
        second_ready && !dial_addr2.is_empty(),
        "instance 2 never ready"
    );
    assert!(
        spawn_tag(&mut terms, &bin, &persist_dir, 3, &[dial_addr1, dial_addr2]),
        "spawn 3"
    );
    assert!(tile_three(GAME_TITLES).is_ok(), "tile game windows");

    let converged = wait_until(TIMEOUT_SECS, || {
        terms
            .iter()
            .all(|g| log_contains(&g.log, "connected, running indefinitely"))
            && terms.iter().all(|g| log_contains(&g.log, "tick lobby="))
            && terms.iter().all(|g| log_contains(&g.log, "lobby=alpha"))
            && terms
                .iter()
                .all(|g| log_contains(&g.log, "accounting for remote owner="))
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

    let auto_ok = wait_until(180, || {
        OWN_IDS.iter().enumerate().all(|(i, tag)| {
            terms
                .get(i)
                .is_some_and(|g| local_count(&g.log, *tag).is_some_and(|c| c >= 15))
        })
    })
    .await;
    tokio::time::sleep(Duration::from_secs(15)).await;
    let (mesh_ok, mesh_lines) = mesh_report(&terms);
    let mesh_ok = auto_ok && mesh_ok;

    let mut checks = Vec::new();
    soft(&mut checks, "mesh-counts", mesh_ok, mesh_lines.join("; "));
    checks.push(color_report(&terms));
    checks.extend(move_report(&terms));

    let recording_start = Instant::now();
    let clip_path = persist_dir.join("clip.mp4");
    let video = start_record(CLIP_SECS, &clip_path).and_then(|child| {
        std::thread::sleep(Duration::from_secs(CLIP_SECS));
        finish_record(child, &clip_path)
    });
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
        "clicker local-mesh lobby={LOBBY} · {} converged={converged} {} all_ok={all_ok}\n{}\n{}\nlogs: {} · build {} s · recording {} s · total {} s",
        check_emoji(converged),
        check_emoji(all_ok),
        mesh_lines.join("\n"),
        check_lines.join("\n"),
        persist_dir.display(),
        fmt_secs(build_elapsed),
        fmt_secs(recording_elapsed),
        fmt_secs(total_elapsed)
    );
    match send_video_file(&creds, &clip, &caption) {
        Ok(message) => {
            println!(
                "telegram video sent: message_id={message} clip={}",
                clip.display()
            );
        }
        Err(err) => {
            assert!(false, "telegram send failed: {err} clip={}", clip.display());
        }
    }
    assert!(converged, "no converge: {}", persist_dir.display());
    for c in &checks {
        assert!(c.ok, "soft check {} failed: {}", c.name, c.detail);
    }
}
