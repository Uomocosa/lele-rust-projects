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
    std::fs::create_dir_all(&instance_dir)?;
    let log_path = instance_dir.join("app.log");
    let log_file = File::create(&log_path)?;

    let title = format!("freenet-example-{index}");
    let cmdline = build_shell(bin, &run_dir.contract_params, &log_path);

    let mut cmd = Command::new("xterm");
    cmd.env("RUST_LOG", "warn,freenet_example=info")
        .arg("-title")
        .arg(&title)
        .arg("-e")
        .arg("bash")
        .arg("-lc")
        .arg(&cmdline)
        .stdout(log_file.try_clone()?)
        .stderr(File::create(instance_dir.join("xterm.err"))?);
    let child = cmd.spawn().map_err(|e| {
        Error::Spawn(format!(
            "spawning xterm instance {index} with {}: {e}",
            bin.display()
        ))
    })?;
    Ok(instance::Instance {
        index,
        pid: child.id(),
        title,
        log_path,
    })
}

// needed helper:
fn build_shell(bin: &Path, contract_params: &str, log_path: &Path) -> String {
    format!(
        "exec '{}' --standalone --mainnet-client --contract-params '{}' 2>&1 | tee '{}'; \
         printf '\\nE2E_EXITED_%s\\n' \"$?\"",
        bin.display(),
        contract_params,
        log_path.display()
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::build_shell;

    #[test]
    fn test_usage() {
        let cmd = build_shell(
            &PathBuf::from("/tmp/bin"),
            "abcd",
            &PathBuf::from("/tmp/app.log"),
        );
        assert!(cmd.contains("--standalone"));
        assert!(cmd.contains("--mainnet-client"));
        assert!(cmd.contains("--contract-params 'abcd'"));
        assert!(cmd.contains("E2E_EXITED"));
    }
}
