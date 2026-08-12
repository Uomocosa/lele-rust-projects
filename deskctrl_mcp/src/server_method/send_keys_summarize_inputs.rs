use crate::KeyboardInput;

pub fn summarize_inputs(inputs: &[KeyboardInput]) -> String {
    if inputs.is_empty() {
        return "sent keys".to_string();
    }
    let mut parts = Vec::with_capacity(inputs.len());
    for input in inputs {
        parts.push(match input {
            KeyboardInput::Tap { key } => format!("tap {}", key.as_str()),
            KeyboardInput::Hold { key, duration_ms } => {
                format!("hold {} for {duration_ms}ms", key.as_str())
            }
            KeyboardInput::Chord { keys } => {
                let names: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();
                format!("chord {}", names.join("+"))
            }
            KeyboardInput::Delay { duration_ms } => format!("delay {duration_ms}ms"),
            KeyboardInput::Text { text } => format!("type {text:?}"),
        });
    }
    format!("sent: {}", parts.join("; "))
}

#[cfg(test)]
mod tests {
    use crate::{KeyboardInput, KeyboardKey};

    use super::summarize_inputs;

    #[test]
    fn test_usage() {
        let inputs = vec![KeyboardInput::Tap {
            key: KeyboardKey("a".to_string()),
        }];
        assert!(summarize_inputs(&inputs).contains("tap a"));
        assert!(summarize_inputs(&[]).contains("sent keys"));
    }
}
