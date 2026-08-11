use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecordVideoParams {
    /// X11 window id to record (optional; whole screen when omitted).
    #[serde(default)]
    pub window_id: Option<String>,
    /// Owning process id to record (optional).
    #[serde(default)]
    pub pid: Option<u32>,
    /// Case-insensitive substring of the window title to record (optional).
    #[serde(default)]
    pub title: Option<String>,
    /// Stop the current recording, send the video to Telegram, and return its caption.
    /// Defaults to false (start).
    #[serde(default)]
    pub stop: bool,
    /// Optional summary text used as the Telegram video caption when stopping.
    #[serde(default)]
    pub summary: Option<String>,
    /// Send a step-by-step message (here: the finished video) to Telegram. Default true.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::RecordVideoParams;

    #[test]
    fn test_usage() {
        let start: RecordVideoParams = serde_json::from_str("{}").unwrap();
        assert!(!start.stop && start.send_to_telegram);
        assert!(start.window_id.is_none() && start.summary.is_none());

        let stop: RecordVideoParams = serde_json::from_str(r#"{"stop":true}"#).unwrap();
        assert!(stop.stop);
    }
}
