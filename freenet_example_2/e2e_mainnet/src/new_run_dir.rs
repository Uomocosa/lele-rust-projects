use std::path::PathBuf;

use chrono::Utc;

use crate::Error;
use crate::new_contract_params;
use crate::run_dir;

pub fn new_run_dir(label: &str) -> Result<run_dir::RunDir, Error> {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| {
            Error::Config("cannot locate freenet_example_2 crate directory".to_string())
        })?
        .join(".local-run");
    std::fs::create_dir_all(&root_dir)?;
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let root = root_dir.join(format!("{stamp}-{label}"));
    std::fs::create_dir_all(&root)?;
    Ok(run_dir::RunDir {
        root,
        contract_params: new_contract_params::new_contract_params(),
        label: label.to_string(),
    })
}

// no test_usage necessary
