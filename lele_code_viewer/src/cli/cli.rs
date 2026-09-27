use std::path::PathBuf;

use clap::Parser;

use crate::cli;

#[derive(Parser)]
#[command(
    name = "lele-code-viewer",
    about = "Mobile-first read-only Rust source, file-tree and dependency-tree viewer"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub crate_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: cli::Command,
}

// no test_usage necessary
