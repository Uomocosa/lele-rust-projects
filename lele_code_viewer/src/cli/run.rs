use std::path::PathBuf;

use clap::Parser;

use crate::Error;
use crate::cli;
use crate::export;
use crate::index;
use crate::project;
use crate::server;

pub fn run() -> Result<(), Error> {
    let parsed = cli::Cli::parse();
    let dir = parsed.crate_dir.unwrap_or_else(|| PathBuf::from("."));
    match parsed.command {
        cli::Command::Check => {
            let idx = index::build_index(&dir)?;
            println!(
                "lele_code_viewer: indexed {} items in {} files ({} markdown)",
                idx.items.len(),
                idx.rust_files.len(),
                idx.markdown_files.len()
            );
            Ok(())
        }
        cli::Command::Export { out } => {
            let idx = index::build_index(&dir)?;
            export::export_site(&idx, &out)?;
            println!("lele_code_viewer: exported to {}", out.display());
            Ok(())
        }
        cli::Command::Graph { out } => {
            let idx = index::build_index(&dir)?;
            export::export_graph(&idx, &out)?;
            println!(
                "lele_code_viewer: wrote tree.json + groups.json ({} groups) to {}",
                idx.item_graph.groups.len(),
                out.display()
            );
            Ok(())
        }
        cli::Command::Serve {
            bind,
            roots,
            watch,
            settings,
            no_self_update,
        } => {
            let roots = if roots.is_empty() {
                default_roots()
            } else {
                roots
            };
            let registry = project::Registry {
                roots,
                watch,
                ..project::Registry::default()
            };
            let runtime = tokio::runtime::Runtime::new()?;
            let options = server::ServeOptions {
                settings_path: settings.unwrap_or_else(project::settings_path),
                self_update: !no_self_update,
            };
            runtime.block_on(server::serve(registry, &bind, options))
        }
    }
}

// needed helper: scan $HOME by default, falling back to the current directory
fn default_roots() -> Vec<PathBuf> {
    match std::env::var_os("HOME") {
        Some(home) => vec![PathBuf::from(home)],
        None => vec![PathBuf::from(".")],
    }
}

// no test_usage necessary
