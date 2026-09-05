use std::path::PathBuf;

use lele_enforce_config_lib::Severity;
use lele_enforce_config_lib::WorkspaceSkipped;
use lele_enforce_config_lib::run;

fn cli_root() -> PathBuf {
    let matches = clap::Command::new("lele-enforce-config")
        .about("Enforce lele devenv.nix config")
        .arg(
            clap::Arg::new("root")
                .default_value(".")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();
    matches
        .get_one::<PathBuf>("root")
        .map_or_else(|| PathBuf::from("."), std::clone::Clone::clone)
}

// needed helper:
fn log_skipped(skipped: &[WorkspaceSkipped]) {
    for workspace in skipped {
        println!("  skipped workspace: {}", workspace.display());
    }
}

fn main() {
    let root = cli_root();
    let (diags, skipped) = match run(&root) {
        Ok(result) => result,
        Err(e) => {
            println!("lele_enforce_config: config error: {e} — blocking commit");
            std::process::exit(1);
        }
    };
    if diags.is_empty() {
        println!(
            "lele_enforce_config: ok ({} workspaces skipped)",
            skipped.len()
        );
        log_skipped(&skipped);
        std::process::exit(0);
    }
    for d in &diags {
        let sev = match d.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        println!("[{sev}] [{}] {}: {}", d.code, d.crate_dir, d.message);
        println!("  hint: {}", d.hint);
    }
    println!(
        "lele_enforce_config: {} diagnostic(s), {} workspace(s) skipped — blocking commit",
        diags.len(),
        skipped.len()
    );
    log_skipped(&skipped);
    std::process::exit(1);
}
