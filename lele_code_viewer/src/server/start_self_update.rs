use std::process::Command;

const UPDATE_UNIT: &str = "lele-code-viewer-update";
const UPDATE_TASK: &str = "devenv tasks run lele:service:update";

pub fn start_self_update() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let output = Command::new("systemd-run")
        .args([
            "--user",
            "--collect",
            "--quiet",
            &format!("--unit={UPDATE_UNIT}"),
            &format!("--working-directory={}", env!("CARGO_MANIFEST_DIR")),
            &format!(
                "--setenv=PATH={home}/.nix-profile/bin:{home}/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
            ),
            "--setenv=CARGO_BUILD_JOBS=2",
            "bash",
            "-lc",
            UPDATE_TASK,
        ])
        .output()
        .map_err(|err| format!("could not run systemd-run: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.contains("already") {
        return Err("an update is already running".to_string());
    }
    Err(format!("systemd-run failed: {stderr}"))
}

// no test_usage necessary
