use std::fs;
use std::path::Path;

use crate::Error;

pub fn has_roster_put(log: &Path) -> Result<bool, Error> {
    let text = fs::read_to_string(log)
        .map_err(|e| Error::Assertion(format!("reading {}: {e}", log.display())))?;
    Ok(text.contains("sending roster Put"))
}

// no test_usage necessary
