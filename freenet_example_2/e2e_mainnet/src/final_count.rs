use std::fs;
use std::path::Path;

use crate::Error;

pub fn final_count(log: &Path) -> Result<Option<u64>, Error> {
    let text = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    Ok(regex::Regex::new(r"tick count=(\d+)")
        .map_err(|e| Error::Assertion(format!("compiling regex: {e}:")))?
        .captures_iter(&strip_ansi(&text))
        .filter_map(|c| c.get(1)?.as_str().parse::<u64>().ok())
        .last())
}

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

    use super::final_count;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("e2e_cnt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        fs::write(
            &p,
            "connected, running indefinitely count=999 owns=999\n\
             tick count=1 owns=1\n\
             tick count=2 owns=2\n\
             tick count=3 owns=3\n",
        )
        .unwrap();
        assert_eq!(final_count(&p).unwrap(), Some(3));
        let p2 = dir.join("b.log");
        fs::write(&p2, "nothing").unwrap();
        assert_eq!(final_count(&p2).unwrap(), None);
    }
}
