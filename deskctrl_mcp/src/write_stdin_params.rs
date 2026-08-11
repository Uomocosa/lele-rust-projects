use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WriteStdinParams {
    /// Managed process ID
    pub pid: u32,
    /// Text to send to the process stdin (newline appended automatically)
    pub text: String,
    /// Send a step-by-step message to Telegram for this action. Default true; set false for
    /// routine input you don't want in the feed.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_true() -> bool {
    true
}
