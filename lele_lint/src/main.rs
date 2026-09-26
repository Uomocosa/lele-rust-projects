use std::path::PathBuf;
use std::process;

use clap::Parser;
use lele_lint::checkers::build_checkers;
use lele_lint::print_checker_list;
use lele_lint::print_diagnostics;
use lele_lint::sync_methods;
use lele_lint::Config;
use lele_lint::Project;
use lele_lint::Severity;

#[derive(Parser)]
#[command(name = "lele_lint", about = "Enforce lele-syntax-rs conventions")]
struct Args {
    #[arg(short, long, default_value = "clippy")]
    error_format: String,

    #[arg(long)]
    checker_list: bool,

    #[arg(long, value_name = "CODE")]
    explain: Option<String>,

    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    #[arg(long = "scan-folder", value_name = "FOLDERS", value_delimiter = ',')]
    scan_folder: Option<Vec<String>>,

    #[arg(long = "sync-methods")]
    sync_methods: bool,

    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if args.checker_list {
        let checkers = build_checkers();
        print_checker_list(&checkers);
        return;
    }

    if let Some(code) = args.explain {
        eprintln!("--explain: not yet implemented for {code}", code = code);
        process::exit(1);
    }

    let mut project = match Project::discover(args.path.as_deref(), args.scan_folder.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("lele_lint: {e}", e = e);
            process::exit(1);
        }
    };

    let config = Config::load(&project.root).unwrap_or_default();

    if let Err(e) = project.apply_layout(&config) {
        eprintln!("lele_lint: {e}", e = e);
        process::exit(1);
    }

    if args.sync_methods {
        if let Err(e) = sync_methods(&project) {
            eprintln!("lele_lint: {e}", e = e);
            process::exit(1);
        }
        return;
    }

    let checkers = build_checkers();

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
