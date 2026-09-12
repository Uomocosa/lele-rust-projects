use std::path::PathBuf;

use lele_hook_resync_lib::Error;
use lele_hook_resync_lib::Report;
use lele_hook_resync_lib::resync;

fn cli_root() -> PathBuf {
    let matches = clap::Command::new("lele-hook-resync")
        .about("Rebuild the combined root .pre-commit-config.yaml from per-crate hook configs")
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

fn main() {
    let root = cli_root();
    let result: Result<Report, Error> = resync(&root);
    match result {
        Ok(report) => {
            let total: usize = report.hooks_per_crate.iter().map(|(_, count)| count).sum();
            println!(
                "lele_hook_resync: wrote {} ({} hooks across {} crates)",
                report.config_path,
                total,
                report.hooks_per_crate.len()
            );
            for (name, count) in &report.hooks_per_crate {
                println!("  {name}: {count} hooks");
            }
            if report.pointer_updated {
                println!("  hook pointer restored to {}", report.config_path);
            }
            std::process::exit(0);
        }
        Err(err) => {
            println!("lele_hook_resync: {err}");
            std::process::exit(1);
        }
    }
}
