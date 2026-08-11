use serde::Deserialize;

/// Selectors for `screenshot`. All optional: with none set, the whole screen is captured.
/// First non-null wins, in order: window_id, pid, title.
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotParams {
    /// X11 window id from list_windows, e.g. "0x03a00004". Captures only that window.
    #[serde(default)]
    pub window_id: Option<String>,
    /// Owning process id, from list_windows. Errors if it matches more than one window.
    #[serde(default)]
    pub pid: Option<u32>,
    /// Case-insensitive substring of the window title. Errors if it matches more than one window.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional Telegram photo caption (HTML), e.g. "freenet clicker state now at 8". Used only
    /// when `send_to_telegram` is true and Telegram is configured; falls back to an auto
    /// summary (target + dimensions) when absent.
    #[serde(default)]
    pub caption: Option<String>,
    /// Send a step-by-step photo message to Telegram for this capture. Default true; set false
    /// for routine/read-only captures you don't want in the feed.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::ScreenshotParams;

    #[test]
    fn test_usage() {
        let empty: ScreenshotParams = serde_json::from_str("{}").unwrap();
        assert!(empty.window_id.is_none() && empty.pid.is_none() && empty.title.is_none());
        assert!(empty.send_to_telegram && empty.caption.is_none());

        let targeted: ScreenshotParams =
            serde_json::from_str(r#"{"window_id":"0x03a00004"}"#).unwrap();
        assert_eq!(targeted.window_id.as_deref(), Some("0x03a00004"));
    }
}
