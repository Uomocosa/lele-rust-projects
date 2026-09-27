use std::path::PathBuf;

use clap::Parser;

use crate::Error;
use crate::cli;
use crate::export;
use crate::index;
use crate::server;

pub fn run() -> Result<(), Error> {
    let parsed = cli::Cli::parse();
    let dir = parsed.crate_dir.unwrap_or_else(|| PathBuf::from("."));
    match parsed.command {
        cli::Command::Check => {
            let idx = index::build_index(&dir)?;
            println!(
                "lele_docs: indexed {} items in {} files ({} markdown)",
                idx.items.len(),
                idx.rust_files.len(),
                idx.markdown_files.len()
            );
            Ok(())
        }
        cli::Command::Export { out } => {
            let idx = index::build_index(&dir)?;
            export::export_site(&idx, &out)?;
            println!("lele_docs: exported to {}", out.display());
            Ok(())
        }
        cli::Command::Serve { bind } => {
            let idx = index::build_index(&dir)?;
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(server::serve(idx, &bind))
        }
    }
}

// no test_usage necessary
