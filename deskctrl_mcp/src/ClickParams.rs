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
}

fn default_button() -> u8 {
    1
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
    }
}
