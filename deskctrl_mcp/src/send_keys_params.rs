use serde::Deserialize;

use crate::KeyboardInput;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendKeysParams {
    /// X11 window id from list_windows, e.g. "0x03a00004".
    pub window_id: String,
    /// A deliberate, bounded sequence of keyboard inputs to send, in order. Each element is one
    /// of: tap (press+release a key), hold (press a key and keep it down for duration_ms, e.g. to
    /// get a run of repeated characters), chord (press a set of keys together and release them all
    /// at once, e.g. ["ctrl","shift","esc"]), delay (pause), or text (literal printable ASCII
    /// typed character by character; "\n" is Enter, "\t" is Tab). Must be non-empty.
    #[serde(default)]
    pub inputs: Vec<KeyboardInput>,
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
        let params: SendKeysParams = serde_json::from_str(
            r#"{"window_id":"0x03a00004","inputs":[{"type":"chord","keys":["ctrl","a"]}]}"#,
        )
        .unwrap();
        assert_eq!(params.window_id, "0x03a00004");
        assert_eq!(params.inputs.len(), 1);
        assert!(params.send_to_telegram);
        assert!(params.note.is_none());
    }
}
