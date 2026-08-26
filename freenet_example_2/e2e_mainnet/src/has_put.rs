use std::fs;
use std::path::Path;

use crate::Error;

pub fn has_put(log: &Path) -> Result<bool, Error> {
    let text = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    Ok(strip_ansi(&text).contains("contract deployed"))
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

    use super::has_put;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("e2e_put_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        fs::write(
            &p,
            "\u{1b}[31m[1] \u{1b}[33mtarget freenet_example\u{1b}[0m key=abc contract deployed\n",
        )
        .unwrap();
        assert!(has_put(&p).unwrap());
        let p2 = dir.join("b.log");
        fs::write(&p2, "no deploy here").unwrap();
        assert!(!has_put(&p2).unwrap());
    }
}
