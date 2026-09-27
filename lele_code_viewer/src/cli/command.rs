use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Check,
    Export {
        #[arg(long)]
        out: PathBuf,
    },
    Serve {
        #[arg(long, default_value = "0.0.0.0:8787")]
        bind: String,
        #[arg(long = "roots")]
        roots: Vec<PathBuf>,
        #[arg(long)]
        watch: bool,
    },
}

// no test_usage necessary
