use std::fs;
use std::path::Path;

use chrono::NaiveDateTime;
use regex::Regex;

use crate::Error;
use crate::tick_sample;

pub fn read_trace(log: &Path) -> Result<Vec<tick_sample::TickSample>, Error> {
    let text = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    let clean = strip_ansi(&text);

    let ts_re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap();
    let count_re = Regex::new(r"count=(\d+)").unwrap();
    let owns_re = Regex::new(r"owns=(\d+)").unwrap();

    let mut out = Vec::new();
    let mut fallback: u64 = 0;
    for line in clean.lines() {
        let Some(ct) = count_re
            .captures(line)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u64>().ok())
        else {
            continue;
        };
        let secs = match ts_re.find(line) {
            Some(m) => NaiveDateTime::parse_from_str(m.as_str(), "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc().timestamp() as u64)
                .unwrap_or_else(|| {
                    fallback += 1;
                    fallback
                }),
            None => {
                fallback += 1;
                fallback
            }
        };
        let owns = owns_re
            .captures(line)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        out.push(tick_sample::TickSample {
            secs,
            count: ct,
            owns,
        });
    }
    Ok(out)
}

// needed helper:
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_escape = false;
    for ch in input.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\u{1b}' {
            in_escape = true;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::read_trace;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("e2e_tr_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        fs::write(
            &p,
            "2026-08-26T11:56:31Z INFO freenet_example_2: tick count=1 owns=1\n\
             2026-08-26T11:56:32Z INFO freenet_example_2: tick count=2 owns=2\n",
        )
        .unwrap();
        let t = read_trace(&p).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].count, 1);
        assert_eq!(t[0].owns, 1);
        assert!(t[0].secs > 1_700_000_000);
        assert!(t[1].secs > t[0].secs);
    }
}
