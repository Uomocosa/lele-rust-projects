use std::fs;
use std::path::Path;

use regex::Regex;

use crate::Error;
use crate::strip_ansi;

pub fn snapshot_x_range(log: &Path) -> Result<(f64, f64), Error> {
    let raw = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    let text = strip_ansi::strip_ansi(&raw);
    let re = Regex::new(r#"sending snapshot.*\bx=([-0-9.eE]+)"#)
        .map_err(|e| Error::Assertion(format!("regex build: {e:?}")))?;
    let mut xs: Vec<f64> = Vec::new();
    for cap in re.captures_iter(&text) {
        if let Ok(x) = cap[1].parse::<f64>() {
            xs.push(x);
        }
    }
    if xs.is_empty() {
        return Ok((0.0, 0.0));
    }
    let min = xs.iter().cloned().fold(f64::MAX, f64::min);
    let max = xs.iter().cloned().fold(f64::MIN, f64::max);
    Ok((min, max))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::snapshot_x_range;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir();
        let p = dir.join("ma_snap.log");
        fs::write(
            &p,
            "sending snapshot player_id=1 x=0.0\nsending snapshot player_id=1 x=42.5\n",
        )
        .unwrap();
        let (min, max) = snapshot_x_range(&p).unwrap();
        assert_eq!(min, 0.0);
        assert_eq!(max, 42.5);
    }
}
