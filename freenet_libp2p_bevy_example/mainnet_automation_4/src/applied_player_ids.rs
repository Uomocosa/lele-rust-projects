use std::collections::HashSet;
use std::fs;
use std::path::Path;

use regex::Regex;

use crate::Error;
use crate::strip_ansi;

pub fn applied_player_ids(log: &Path) -> Result<HashSet<String>, Error> {
    let raw = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    let text = strip_ansi::strip_ansi(&raw);
    let re = Regex::new(r#"received peer input.*\bplayer_id=([0-9a-fA-F]{64})"#)
        .map_err(|e| Error::Assertion(format!("regex build: {e:?}")))?;
    Ok(re
        .captures_iter(&text)
        .map(|cap| cap[1].to_string())
        .collect())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::applied_player_ids;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("ma_peers_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        let id = "ab".repeat(32);
        fs::write(
            &p,
            format!(
                "\u{1b}[2mDEBUG\u{1b}[0m received peer input \u{1b}[3mplayer_id\u{1b}[0m\u{1b}[2m=\u{1b}[0m{id}\n"
            ),
        )
        .unwrap();
        let peers = applied_player_ids(&p).unwrap();
        assert!(peers.contains(id.as_str()));
    }
}
