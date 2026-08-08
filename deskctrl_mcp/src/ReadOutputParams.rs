use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadOutputParams {
    /// Managed process ID
    pub pid: u32,
    /// How long to wait for new output (milliseconds)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    200
}
