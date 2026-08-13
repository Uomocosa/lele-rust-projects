use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LaunchGameParams {
    /// Path to the game executable; defaults to the self-hosted build output
    pub exe: Option<String>,
    /// Directory for the persistent identity (recommended on both machines)
    pub identity_dir: String,
    /// Fixed freenet p2p port (use a distinct port per machine)
    pub p2p_port: u16,
    /// Log file path; defaults to fbx_game.log in the current dir
    pub log_file: Option<String>,
}
