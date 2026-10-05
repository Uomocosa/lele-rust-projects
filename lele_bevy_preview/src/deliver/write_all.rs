use std::collections::BTreeMap;
use std::path::Path;

use crate::Error;
use crate::deliver;
use crate::deliver::basic::structs::{Capture, Manifest};

pub fn write_all(dir: &Path, crate_name: &str, captures: &[Capture]) -> Result<(), Error> {
    let manifest = Manifest {
        crate_name: crate_name.to_string(),
        artifacts: captures
            .iter()
            .map(|capture| {
                (
                    capture.artifact.fingerprint.clone(),
                    capture.artifact.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    };
    let json =
        serde_json::to_string_pretty(&manifest).map_err(|error| Error::Io(error.to_string()))?;
    std::fs::create_dir_all(dir).map_err(|error| Error::Io(error.to_string()))?;
    std::fs::write(dir.join(deliver::basic::constants::MANIFEST_FILE), json)
        .map_err(|error| Error::Io(error.to_string()))?;
    std::fs::write(
        dir.join(deliver::basic::constants::CHANGES_FILE),
        deliver::changes_markdown::changes_markdown(captures),
    )
    .map_err(|error| Error::Io(error.to_string()))
}
// no test_usage necessary
