use std::path::PathBuf;

/// Command-line arguments for `freenet-libp2p-bevy-example-4`.
#[derive(clap::Parser)]
#[command(name = "freenet-libp2p-bevy-example-4")]
pub struct Cli {
    /// Directory holding this instance's identity keypair (default:
    /// `$HOME/.local/share/freenet-libp2p-bevy-example-4`). Two instances started on one
    /// machine without this flag share that directory, so they load the same keypair, derive
    /// the same PlayerId, and collapse into a single roster entry — pass a distinct directory
    /// per instance when running more than one locally.
    #[arg(long)]
    pub identity_dir: Option<PathBuf>,

    /// Contract parameters distinguishing this roster from production (omit for the real production roster)
    #[arg(long)]
    pub contract_params: Option<String>,

    /// Run an isolated local Freenet node instead of joining the public mainnet
    #[arg(long)]
    pub freenet_local: bool,

    /// Dial this gateway directly instead of using public mainnet discovery
    #[arg(long)]
    pub freenet_gateway: Option<String>,
}
