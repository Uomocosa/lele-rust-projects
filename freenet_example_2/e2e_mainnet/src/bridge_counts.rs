use std::fs;
use std::path::Path;

use crate::Error;

pub fn bridge_counts(logs: &[&Path]) -> Result<(usize, usize), Error> {
    let mut splits = 0;
    let mut merges = 0;
    for log in logs {
        let text = fs::read_to_string(log)
            .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
        let clean = strip_ansi(&text);
        splits += clean.matches("bridge: split suspected").count();
        merges += clean.matches("bridge: merged").count();
    }
    Ok((splits, merges))
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

    use super::bridge_counts;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("e2e_brg_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        fs::write(
            &p,
            "bridge: split suspected\n\
             bridge: resubscribe attempt\n\
             bridge: re-put attempt\n\
             bridge: merged via resubscribe\n\
             bridge: split suspected\n\
             bridge: merged via re-put\n",
        )
        .unwrap();
        let (splits, merges) = bridge_counts(&[&p]).unwrap();
        assert_eq!(splits, 2);
        assert_eq!(merges, 2);
    }
}
