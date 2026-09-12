use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MIN_X: i32 = 50;
const MAX_X: i32 = 750;
const MIN_Y: i32 = 50;
const MAX_Y: i32 = 400;

/// # Errors
/// Returns an error if the window cannot be found or the synthetic mouse
/// cannot be moved or clicked.
pub fn drive_random(title: &str, clicks: u32) -> Result<(), String> {
    let wid = window_id(title)?;
    let mut state = seed_now();
    for _ in 0..clicks {
        state = next_state(state);
        let x = next_bounded(state, MIN_X, MAX_X);
        state = next_state(state);
        let y = next_bounded(state, MIN_Y, MAX_Y);
        mouse_move(&wid, x, y)?;
        std::thread::sleep(Duration::from_millis(250));
        mouse_click()?;
        std::thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}

// needed helper: time-seeded LCG state starter without panicking clocks
fn seed_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9E37_79B9_7F4A_7C15, |elapsed| {
            elapsed
                .as_secs()
                .wrapping_mul(1_000_000_009)
                .wrapping_add(u64::from(elapsed.subsec_nanos()))
        })
}

// needed helper: deterministic LCG step with explicit wrapping
const fn next_state(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

// needed helper: maps state into min..=max without arithmetic panics
fn next_bounded(state: u64, min: i32, max: i32) -> i32 {
    let Some(span) = max.checked_sub(min) else {
        return min;
    };
    let Ok(span_u64) = u64::try_from(span) else {
        return min;
    };
    let Some(slots) = span_u64.checked_add(1) else {
        return min;
    };
    let Some(offset) = state.checked_rem(slots) else {
        return min;
    };
    let Ok(offset_i32) = i32::try_from(offset) else {
        return min;
    };
    min.checked_add(offset_i32).unwrap_or(min)
}

// needed helper: resolves the X window id by substring title match
fn window_id(title: &str) -> Result<String, String> {
    let output = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", title])
        .output()
        .map_err(|err| format!("xdotool search {title}: {err}"))?;
    if !output.status.success() {
        return Err(format!("xdotool cannot find window {title}"));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .next()
        .map(str::to_string)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| format!("xdotool found no id for {title}"))
}

// needed helper: moves the real cursor to window-relative coordinates
fn mouse_move(wid: &str, x: i32, y: i32) -> Result<(), String> {
    let status = Command::new("xdotool")
        .args(["mousemove", "--window", wid, &x.to_string(), &y.to_string()])
        .status()
        .map_err(|err| format!("xdotool mousemove: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("xdotool mousemove failed".to_string())
    }
}

// needed helper: sends a real left click at the current pointer position
fn mouse_click() -> Result<(), String> {
    let status = Command::new("xdotool")
        .args(["click", "1"])
        .status()
        .map_err(|err| format!("xdotool click failed: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("xdotool click failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{drive_random, next_bounded, next_state, seed_now};

    #[test]
    fn test_usage() {
        assert!(drive_random("no-such-clicker-window-xyz", 1).is_err());
        assert!(drive_random("no-such-clicker-window-xyz", 0).is_err());
        let first = next_state(seed_now());
        let x = next_bounded(first, 50, 750);
        assert!((50..=750).contains(&x));
        let y = next_bounded(next_state(first), 50, 400);
        assert!((50..=400).contains(&y));
        assert_eq!(next_bounded(0, 10, 10), 10);
    }
}
