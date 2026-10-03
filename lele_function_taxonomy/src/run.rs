use std::path::PathBuf;
use std::process::Command;

use crate::config;
use crate::toolchain;

pub fn run(manifest_path: Option<PathBuf>) -> i32 {
    let manifest = manifest_path.unwrap_or_else(|| PathBuf::from("./Cargo.toml"));
    let Ok(manifest) = std::fs::canonicalize(&manifest) else {
        eprintln!("TAX002: manifest not found: {}", manifest.display());
        return 1;
    };

    let config = match config::load(&manifest) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };

    if !config.has_honest_boundary() {
        println!("no honest boundaries configured — nothing to check");
        return 0;
    }

    if let Err(e) = config::validate(&manifest, &config) {
        eprintln!("{e}");
        return 1;
    }

    let driver = if let Some(driver) = toolchain::driver_path() {
        driver
    } else if let Some(driver) = toolchain::build_driver() {
        driver
    } else {
        eprintln!("TAX002: could not locate or build the taxonomy driver");
        return 1;
    };

    let config_path = config::lele_toml_path(&manifest);
    let target_dir = toolchain::target_dir();

    let lib_path = {
        let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let sysroot_lib = toolchain::sysroot_lib();
        if existing.is_empty() {
            sysroot_lib.to_string_lossy().into_owned()
        } else {
            format!("{}:{}", sysroot_lib.display(), existing)
        }
    };

    let mut command = Command::new(toolchain::pinned_cargo());
    command
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--lib")
        .arg("--bins")
        .env("RUSTC_WORKSPACE_WRAPPER", &driver)
        .env("LELE_TAXONOMY_CONFIG", &config_path)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("LD_LIBRARY_PATH", lib_path)
        .env("CARGO_BUILD_JOBS", "6");
    toolchain::apply_pinned_env(&mut command);
    let status = command.status();

    match status {
        Ok(status) if status.success() => 0,
        Ok(_) => 1,
        Err(e) => {
            eprintln!("TAX002: failed to run cargo check: {e}");
            1
        }
    }
}

// no test_usage necessary
