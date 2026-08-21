use std::fs::File;
use std::path::Path;
use std::process::Command;

use crate::Error;
use crate::instance;
use crate::run_dir;

pub fn spawn_one(
    index: usize,
    bin: &Path,
    run_dir: &run_dir::RunDir,
) -> Result<instance::Instance, Error> {
    let instance_dir = run_dir.root.join(format!("instance-{index}"));
    let identity_dir = instance_dir.join("identity");
    let log_path = instance_dir.join("app.log");
    std::fs::create_dir_all(&identity_dir)?;

    let log_file = File::create(&log_path)?;
    let err_file = log_file.try_clone()?;

    let mut cmd = Command::new(bin);
    cmd.env("RUST_LOG", "warn,roster=trace,p2p=debug,freenet_bevy=debug")
        .env("RUST_BACKTRACE", "1")
        .arg("--identity-dir")
        .arg(&identity_dir)
        .arg("--contract-params")
        .arg(&run_dir.contract_params)
        .stdout(log_file)
        .stderr(err_file);
    let child = cmd.spawn().map_err(|e| {
        Error::Spawn(format!(
            "spawning instance {index} from {}: {e}",
            bin.display()
        ))
    })?;
    Ok(instance::Instance {
        index,
        pid: child.id(),
        log_path,
        identity_dir,
    })
}

// no test_usage necessary
