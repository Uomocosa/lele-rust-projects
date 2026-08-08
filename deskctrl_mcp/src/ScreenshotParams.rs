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
}

#[cfg(test)]
mod tests {
    use crate::ScreenshotParams;

    #[test]
    fn test_usage() {
        let empty: ScreenshotParams = serde_json::from_str("{}").unwrap();
        assert!(empty.window_id.is_none() && empty.pid.is_none() && empty.title.is_none());

        let targeted: ScreenshotParams =
            serde_json::from_str(r#"{"window_id":"0x03a00004"}"#).unwrap();
        assert_eq!(targeted.window_id.as_deref(), Some("0x03a00004"));
    }
}
