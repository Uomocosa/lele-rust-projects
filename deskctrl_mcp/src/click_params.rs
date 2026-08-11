use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClickParams {
    /// X11 window id from list_windows, e.g. "0x03a00004".
    pub window_id: String,
    /// X offset in pixels from the window's top-left corner — the same coordinates as a
    /// `screenshot {window_id}` image.
    pub x: i16,
    /// Y offset in pixels from the window's top-left corner.
    pub y: i16,
    /// Mouse button: 1 left, 2 middle, 3 right. Defaults to 1.
    #[serde(default = "default_button")]
    pub button: u8,
    /// Optional template for the Telegram message, e.g. "clicking 'Increment button',
    /// expected in the image: freenet clicker state now at 8". Shown only when
    /// `send_to_telegram` is true and Telegram is configured.
    #[serde(default)]
    pub note: Option<String>,
    /// Send a step-by-step message to Telegram for this action. Default true; set false for
    /// routine clicks you don't want in the feed.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_button() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::ClickParams;

    #[test]
    fn test_usage() {
        let params: ClickParams =
            serde_json::from_str(r#"{"window_id":"0x03a00004","x":600,"y":732}"#).unwrap();
        assert_eq!(params.button, 1);
        assert_eq!(params.x, 600);
        // The rule: notification defaults ON; the agent opts out for routine clicks.
        assert!(params.send_to_telegram);
        assert!(params.note.is_none());
    }
}
