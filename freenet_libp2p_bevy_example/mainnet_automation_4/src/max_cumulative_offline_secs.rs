use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::DateTime;
use regex::Regex;

use crate::Error;
use crate::strip_ansi;

const SPAWN: &str = r#"spawning box for player\s+player="?([0-9a-f]+)"?"#;
const DESPAWN: &str = r#"despawning box for departed player\s+player="?([0-9a-f]+)"?"#;

/// Tracks every despawn->respawn gap per player across `log` and returns the largest per-player
/// CUMULATIVE offline time in seconds. A despawn that never respawns by the end of the log is
/// charged up to the last logged timestamp so a genuinely-lost box cannot slip through.
pub fn max_cumulative_offline_secs(log: &Path) -> Result<f64, Error> {
    let text = read_stripped(log)?;
    let spawn_re = Regex::new(SPAWN).map_err(err_re)?;
    let despawn_re = Regex::new(DESPAWN).map_err(err_re)?;

    let mut pending: HashMap<String, f64> = HashMap::new();
    let mut total: HashMap<String, f64> = HashMap::new();
    let mut last_ts: Option<f64> = None;

    for line in text.lines() {
        let Some(ts_ms) = line_ts(line) else {
            continue;
        };
        last_ts = Some(ts_ms);
        if let Some(cap) = spawn_re.captures(line) {
            let key = cap[1].to_string();
            if let Some(t1) = pending.remove(&key) {
                let gap = total.entry(key).or_insert(0.0);
                *gap += ts_ms - t1;
            }
        } else if let Some(cap) = despawn_re.captures(line) {
            let key = cap[1].to_string();
            pending.insert(key, ts_ms);
        }
    }
    for (key, t1) in pending {
        if let Some(last) = last_ts {
            let gap = total.entry(key).or_insert(0.0);
            *gap += last - t1;
        }
    }
    let max_ms = total.values().cloned().fold(0.0, f64::max);
    Ok(max_ms / 1000.0)
}

fn read_stripped(log: &Path) -> Result<String, Error> {
    let raw = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    Ok(strip_ansi::strip_ansi(&raw))
}

fn line_ts(line: &str) -> Option<f64> {
    let token = line.split_whitespace().next()?;
    let dt = DateTime::parse_from_rfc3339(token).ok()?;
    Some(dt.timestamp_millis() as f64)
}

// needed helper:
fn err_re<E: std::fmt::Debug>(e: E) -> Error {
    Error::Assertion(format!("regex build: {e:?}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::max_cumulative_offline_secs;

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn write_log(lines: &[&str]) -> std::path::PathBuf {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ma_flick_{}_{n}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        fs::write(&p, lines.join("\n")).unwrap();
        p
    }

    #[test]
    fn test_usage() {
        let p = write_log(&[
            "2026-08-21T00:00:00.000Z DEBUG roster: despawning box for departed player player=\"aaaa\"",
            "2026-08-21T00:00:04.000Z DEBUG roster: spawning box for player player=\"aaaa\" x=10",
        ]);
        assert_eq!(max_cumulative_offline_secs(&p).unwrap(), 4.0);
    }

    #[test]
    fn test_cumulative_repeated_gaps() {
        let p = write_log(&[
            "2026-08-21T00:01:00.000Z DEBUG roster: despawning box for departed player player=\"bbbb\"",
            "2026-08-21T00:01:06.000Z DEBUG roster: spawning box for player player=\"bbbb\" x=10",
            "2026-08-21T00:01:10.000Z DEBUG roster: despawning box for departed player player=\"bbbb\"",
            "2026-08-21T00:01:16.000Z DEBUG roster: spawning box for player player=\"bbbb\" x=20",
        ]);
        assert_eq!(max_cumulative_offline_secs(&p).unwrap(), 12.0);
    }

    #[test]
    fn test_open_despawn_charges_to_end() {
        let p = write_log(&[
            "2026-08-21T00:02:00.000Z DEBUG roster: despawning box for departed player player=\"cccc\"",
            "2026-08-21T00:02:20.000Z DEBUG roster: spawning box for player player=\"aaaa\" x=10",
        ]);
        assert_eq!(max_cumulative_offline_secs(&p).unwrap(), 20.0);
    }
}
