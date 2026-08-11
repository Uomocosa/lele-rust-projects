use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendKeysParams {
    /// X11 window id from list_windows, e.g. "0x03a00004".
    pub window_id: String,
    /// Literal text to type into the focused window, character by character. "\n" is Enter,
    /// "\t" is Tab. Provide exactly one of text or keys.
    #[serde(default)]
    pub text: Option<String>,
    /// Named keys or shortcuts to press, e.g. "Enter", "Tab", "BackSpace", "Escape", "F5",
    /// "Ctrl+A", "Alt+Tab", "Ctrl+Shift+Esc". Provide exactly one of text or keys.
    #[serde(default)]
    pub keys: Option<String>,
    /// Optional template for the Telegram message, e.g. "typing 'ls' in xterm, expected in
    /// the image: the prompt shows the typed command". Shown only when `send_to_telegram` is
    /// true and Telegram is configured.
    #[serde(default)]
    pub note: Option<String>,
    /// Send a step-by-step message to Telegram for this action. Default true; set false for
    /// routine input you don't want in the feed.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::SendKeysParams;

    #[test]
    fn test_usage() {
        let params: SendKeysParams =
            serde_json::from_str(r#"{"window_id":"0x03a00004","keys":"Ctrl+A"}"#).unwrap();
        assert_eq!(params.keys.as_deref(), Some("Ctrl+A"));
        assert!(params.text.is_none());
        assert!(params.send_to_telegram);
        assert!(params.note.is_none());
    }
}
