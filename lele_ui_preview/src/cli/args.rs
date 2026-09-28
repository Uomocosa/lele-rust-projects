use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "lele-ui-preview",
    about = "Crawl an app's UI states into <crate>/ui_preview (configured by [ui_preview] in lele.toml)"
)]
pub struct Args {
    pub crate_dir: PathBuf,
}

// no test_usage necessary
