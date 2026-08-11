use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SpawnParams {
    /// The executable to run (name or full path)
    pub cmd: String,
    /// Command-line arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory (optional; defaults to current dir)
    pub cwd: Option<String>,
    /// Extra environment variables to set for the child process
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Send a step-by-step message to Telegram for this action. Default true; set false for
    /// routine/background spawns you don't want in the feed.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_true() -> bool {
    true
}
