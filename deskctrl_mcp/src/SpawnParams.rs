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
}
