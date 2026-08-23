use std::fs;
use std::path::Path;

use crate::Error;

const PATTERNS: [&str; 3] = [
    "error exited with error",
    "panicked",
    "update confirmation timed out",
];

pub fn collect_error_sigs(logs: &[&Path]) -> Result<Vec<String>, Error> {
    let mut found = Vec::new();
    for log in logs {
        let text = fs::read_to_string(log)
            .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
        for pat in PATTERNS {
            if text.contains(pat) {
                found.push(format!("{}: {pat}", log.display()));
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::collect_error_sigs;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("ma_err_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.log");
        fs::write(&p, "panicked at main\n").unwrap();
        let found = collect_error_sigs(&[p.as_path()]).unwrap();
        assert!(!found.is_empty());
    }
}
