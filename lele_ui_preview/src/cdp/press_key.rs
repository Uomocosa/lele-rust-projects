use serde_json::json;

use crate::Error;
use crate::cdp;

pub fn press_key(session: &mut cdp::Session, key: &str) -> Result<(), Error> {
    let code = virtual_key_code(key);
    for kind in ["keyDown", "keyUp"] {
        cdp::call(
            session,
            "Input.dispatchKeyEvent",
            &json!({
                "type": kind,
                "key": key,
                "code": key,
                "windowsVirtualKeyCode": code,
                "nativeVirtualKeyCode": code,
            }),
        )?;
    }
    Ok(())
}

// needed helper: Windows virtual-key codes for the keys apps commonly bind
fn virtual_key_code(key: &str) -> u32 {
    match key {
        "Escape" => 27,
        "Enter" => 13,
        "Tab" => 9,
        "Backspace" => 8,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        _ => key
            .chars()
            .next()
            .map_or(0, |c| u32::from(c.to_ascii_uppercase())),
    }
}

#[cfg(test)]
mod tests {
    use super::virtual_key_code;

    #[test]
    fn test_usage() {
        assert_eq!(virtual_key_code("Escape"), 27);
        assert_eq!(virtual_key_code("k"), 75);
    }
}
