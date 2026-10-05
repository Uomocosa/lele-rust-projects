use std::path::Path;

use crate::Error;
use crate::deliver;

pub fn hash_file(path: &Path) -> Result<String, Error> {
    let bytes = std::fs::read(path).map_err(|error| Error::Io(error.to_string()))?;
    Ok(deliver::hash_bytes::hash_bytes(&bytes))
}
// no test_usage necessary
