use serde::Deserialize;

use crate::KeyboardKey;

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KeyboardInput {
    /// Press and release a single key.
    Tap { key: KeyboardKey },
    /// Press a key and hold it for the given duration, then release. X keyboard auto-repeat
    /// turns a long hold into repeated characters, so use this instead of spelling out a run
    /// of identical taps.
    Hold { key: KeyboardKey, duration_ms: u64 },
    /// Press several keys together (modifiers plus a target) and release them all at once, e.g.
    /// ["ctrl", "shift", "esc"].
    Chord { keys: Vec<KeyboardKey> },
    /// Wait a while without sending any keys.
    Delay { duration_ms: u64 },
    /// Type literal printable ASCII text, character by character. "\n" is Enter, "\t" is Tab.
    Text { text: String },
}

#[cfg(test)]
mod tests {
    use crate::KeyboardInput;

    #[test]
    fn test_usage() {
        let chord: KeyboardInput =
            serde_json::from_str(r#"{"type":"chord","keys":["ctrl","shift","esc"]}"#).unwrap();
        match chord {
            KeyboardInput::Chord { keys } => assert_eq!(keys.len(), 3),
            _ => panic!("expected chord"),
        }
    }
}
