use std::path::PathBuf;
use std::process;

use clap::Parser;
use lele_bevy_lint::checkers::build_checkers;
use lele_lint::print_checker_list;
use lele_lint::print_diagnostics;
use lele_lint::Config;
use lele_lint::Project;
use lele_lint::Severity;

#[derive(Parser)]
#[command(
    name = "lele_bevy_lint",
    about = "Enforce Bevy-specific lele-syntax-rs conventions"
)]
struct Args {
    #[arg(short, long, default_value = "clippy")]
    error_format: String,

    #[arg(long)]
    checker_list: bool,

    #[arg(long, value_name = "CODE")]
    explain: Option<String>,

    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if args.checker_list {
        let config = Config::default();
        let checkers = build_checkers(&config);
        print_checker_list(&checkers);
        return;
    }

    if let Some(code) = args.explain {
        eprintln!("--explain: not yet implemented for {code}", code = code);
        process::exit(1);
    }

    let project = match Project::discover(args.path.as_deref(), None) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("lele_bevy_lint: {e}", e = e);
            process::exit(1);
        }
    };

    let config = Config::load(&project.root).unwrap_or_default();

    let checkers = build_checkers(&config);

    let mut all_diags = Vec::new();
    for checker in &checkers {
        let diags = checker.check(&project);
        all_diags.extend(diags);
    }

    print_diagnostics(&all_diags, &args.error_format);

    let error_count = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();

    if error_count > 0 {
        process::exit(1);
    }
}
