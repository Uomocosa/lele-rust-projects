use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WriteStdinParams {
    /// Managed process ID
    pub pid: u32,
    /// Text to send to the process stdin (newline appended automatically)
    pub text: String,
}
