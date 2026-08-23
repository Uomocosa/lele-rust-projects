use std::path::Path;

use crate::Error;
use crate::instance;
use crate::run_dir;
use crate::spawn_one;

pub fn launch_instances(
    bin: &Path,
    run_dir: &run_dir::RunDir,
    count: usize,
) -> Result<Vec<instance::Instance>, Error> {
    let mut instances = Vec::with_capacity(count);
    for index in 0..count {
        instances.push(spawn_one::spawn_one(index, bin, run_dir)?);
    }
    Ok(instances)
}

// no test_usage necessary
