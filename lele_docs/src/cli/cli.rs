use std::path::PathBuf;

use clap::Parser;

use crate::cli;

#[derive(Parser)]
#[command(
    name = "lele-docs",
    about = "Mobile-first read-only Rust source + markdown browser"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub crate_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: cli::Command,
}

// no test_usage necessary
