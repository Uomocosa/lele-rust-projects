use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PidParam {
    /// Managed process ID returned by spawn_process
    pub pid: u32,
}
