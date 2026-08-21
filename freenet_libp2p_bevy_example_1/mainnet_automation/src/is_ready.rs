use std::fs;
use std::path::Path;

use crate::Error;

pub fn is_ready(log: &Path) -> Result<bool, Error> {
    let text = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    Ok(text.contains("embedded node ready"))
}

// no test_usage necessary
