use std::{collections::HashMap, thread, time::Duration};

use rmcp::model::{CallToolResult, ContentBlock};
use x11rb::{
    connection::Connection as _,
    protocol::{xproto::ConnectionExt as _, xtest::ConnectionExt as _},
    rust_connection::RustConnection,
};

use crate::{Error, SendKeysParams, window_info_method};

const KEY_PRESS_EVENT: u8 = 2;
const KEY_RELEASE_EVENT: u8 = 3;
const FIRST_KEYCODE: u8 = 8;
const KEYCODE_COUNT: u8 = 248;
const INTER_KEY_DELAY_MS: u64 = 10;

const SHIFT_L: u32 = 0xFFE1;
const SHIFT_R: u32 = 0xFFE2;
const CONTROL_L: u32 = 0xFFE3;
const ALT_L: u32 = 0xFFE9;
const SUPER_L: u32 = 0xFFEB;
const META_L: u32 = 0xFFE7;
const RETURN: u32 = 0xFF0D;
const TAB: u32 = 0xFF09;
const BACK_SPACE: u32 = 0xFF08;
const ESCAPE: u32 = 0xFF1B;
const DELETE: u32 = 0xFFFF;
const INSERT: u32 = 0xFF63;
const HOME: u32 = 0xFF50;
const END: u32 = 0xFF57;
const PAGE_UP: u32 = 0xFF55;
const PAGE_DOWN: u32 = 0xFF56;
const LEFT: u32 = 0xFF51;
const UP: u32 = 0xFF52;
const RIGHT: u32 = 0xFF53;
const DOWN: u32 = 0xFF54;
const SPACE: u32 = 0x20;
const F1: u32 = 0xFFBE;

#[derive(Debug)]
struct KeySpec {
    keysym: u32,
    hold: bool,
}

#[derive(Debug, Default)]
struct Keymap {
    plain: HashMap<u32, u8>,
    shifted: HashMap<u32, u8>,
}

pub async fn send_keys(params: SendKeysParams) -> Result<CallToolResult, Error> {
    let window_id = &params.window_id;
    if !window_info_method::is_valid_id(window_id) {
        return Err(Error::Window(format!(
            "invalid window_id {window_id:?}: expected hex like \"0x03a00004\" (see list_windows)"
        )));
    }
    let specs = build_specs(&params)?;
    if specs.is_empty() {
        return Err(Error::Window(
            "nothing to send: provide non-empty text or keys".to_string(),
        ));
    }

    // XTEST synthesizes input at the *root*, so it lands on whatever is topmost. Raise the
    // target first or an overlapping window eats the keystrokes.
    super::raise_window::raise_window(window_id);

    let (conn, _screen) =
        x11rb::connect(None).map_err(|e| Error::Window(format!("connecting to X display: {e}")))?;
    let keymap = keymap(&conn)?;
    type_specs(&conn, &specs, &keymap)?;
    conn.flush()
        .map_err(|e| Error::Window(format!("flushing keystrokes: {e}")))?;

    let summary = match (params.text.as_deref(), params.keys.as_deref()) {
        (Some(text), _) => format!("typed {text:?} in {window_id}"),
        (None, Some(keys)) => format!("pressed {keys} in {window_id}"),
        (None, None) => format!("sent keys to {window_id}"),
    };
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "{summary}; screenshot the window to confirm it took effect"
    ))]))
}

// needed helper:
fn build_specs(params: &SendKeysParams) -> Result<Vec<KeySpec>, Error> {
    match (&params.text, &params.keys) {
        (Some(text), None) => parse_text(text),
        (None, Some(keys)) => parse_keys(keys),
        _ => Err(Error::Window(
            "provide exactly one of text or keys".to_string(),
        )),
    }
}

// needed helper:
fn parse_text(text: &str) -> Result<Vec<KeySpec>, Error> {
    text.chars()
        .map(|c| match c {
            '\n' => Ok(KeySpec {
                keysym: RETURN,
                hold: false,
            }),
            '\t' => Ok(KeySpec {
                keysym: TAB,
                hold: false,
            }),
            ' '..='~' => Ok(KeySpec {
                keysym: c as u32,
                hold: false,
            }),
            _ => Err(Error::Window(format!(
                "unsupported character {c:?}: text is limited to printable ASCII plus \\n and \\t"
            ))),
        })
        .collect()
}

// needed helper:
fn parse_keys(keys: &str) -> Result<Vec<KeySpec>, Error> {
    let parts: Vec<&str> = keys.split('+').map(str::trim).collect();
    let last = parts.len() - 1;
    parts
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let keysym = named_keysym(name).ok_or_else(|| {
                Error::Window(format!(
                    "unknown key {name:?} in {keys:?}: use a modifier (Ctrl, Shift, Alt, Super, \
                     Meta), a named key (Enter, Tab, BackSpace, Escape, Delete, Insert, Home, \
                     End, PageUp, PageDown, arrows, Space, F1-F12) or a single printable ASCII \
                     character"
                ))
            })?;
            Ok(KeySpec {
                keysym,
                hold: i < last,
            })
        })
        .collect()
}

// needed helper:
fn named_keysym(name: &str) -> Option<u32> {
    if name.len() == 1 {
        let c = name.chars().next()?;
        return Some(if c.is_ascii_alphanumeric() {
            c.to_ascii_lowercase() as u32
        } else {
            c as u32
        });
    }
    if let Some(n) = name
        .strip_prefix('F')
        .and_then(|digits| digits.parse::<u32>().ok())
        .filter(|&n| (1..=12).contains(&n))
    {
        return Some(F1 + n - 1);
    }
    match name {
        "Ctrl" | "Control" => Some(CONTROL_L),
        "Shift" => Some(SHIFT_L),
        "Alt" => Some(ALT_L),
        "Super" | "Win" => Some(SUPER_L),
        "Meta" => Some(META_L),
        "Enter" | "Return" => Some(RETURN),
        "Tab" => Some(TAB),
        "BackSpace" | "Backspace" => Some(BACK_SPACE),
        "Escape" | "Esc" => Some(ESCAPE),
        "Delete" | "Del" => Some(DELETE),
        "Insert" | "Ins" => Some(INSERT),
        "Home" => Some(HOME),
        "End" => Some(END),
        "PageUp" => Some(PAGE_UP),
        "PageDown" => Some(PAGE_DOWN),
        "Left" => Some(LEFT),
        "Right" => Some(RIGHT),
        "Up" => Some(UP),
        "Down" => Some(DOWN),
        "Space" => Some(SPACE),
        _ => None,
    }
}

// needed helper:
fn keymap(conn: &RustConnection) -> Result<Keymap, Error> {
    let reply = conn
        .get_keyboard_mapping(FIRST_KEYCODE, KEYCODE_COUNT)
        .map_err(|e| Error::Window(format!("get_keyboard_mapping request: {e}")))?
        .reply()
        .map_err(|e| Error::Window(format!("get_keyboard_mapping reply: {e}")))?;
    Ok(index_keymap(&reply.keysyms, reply.keysyms_per_keycode))
}

// needed helper:
fn index_keymap(keysyms: &[u32], per_keycode: u8) -> Keymap {
    let mut plain = HashMap::new();
    let mut shifted = HashMap::new();
    let per = per_keycode as usize;
    if per == 0 {
        return Keymap { plain, shifted };
    }
    for (i, &keysym) in keysyms.iter().enumerate() {
        if keysym == 0 {
            continue;
        }
        let keycode = FIRST_KEYCODE + (i / per) as u8;
        match i % per {
            0 => {
                plain.entry(keysym).or_insert(keycode);
            }
            1 if !plain.contains_key(&keysym) => {
                shifted.insert(keysym, keycode);
            }
            _ => {}
        }
    }
    Keymap { plain, shifted }
}

// needed helper:
fn resolve_key(keysym: u32, keymap: &Keymap) -> Result<(u8, bool), Error> {
    if let Some(&keycode) = keymap.plain.get(&keysym) {
        return Ok((keycode, false));
    }
    if let Some(&keycode) = keymap.shifted.get(&keysym) {
        return Ok((keycode, true));
    }
    Err(Error::Window(format!(
        "keysym {keysym:#x} is not available on the current keyboard layout"
    )))
}

// needed helper:
fn fake_key(conn: &RustConnection, event: u8, keycode: u8) -> Result<(), Error> {
    conn.xtest_fake_input(event, keycode, 0, 0, 0, 0, 0)
        .map_err(|e| Error::Window(format!("xtest_fake_input: {e}")))?;
    Ok(())
}

// needed helper:
fn type_specs(conn: &RustConnection, specs: &[KeySpec], keymap: &Keymap) -> Result<(), Error> {
    let mut held = Vec::new();
    for spec in specs {
        if spec.hold {
            let (keycode, needs_shift) = resolve_key(spec.keysym, keymap)?;
            if needs_shift {
                return Err(Error::Window(format!(
                    "modifier keysym {:#x} is only reachable via Shift; cannot hold it",
                    spec.keysym
                )));
            }
            fake_key(conn, KEY_PRESS_EVENT, keycode)?;
            held.push(keycode);
        } else {
            let (keycode, needs_shift) = resolve_key(spec.keysym, keymap)?;
            if needs_shift {
                let shift_keycode = keymap
                    .plain
                    .get(&SHIFT_L)
                    .or_else(|| keymap.plain.get(&SHIFT_R))
                    .copied()
                    .ok_or_else(|| {
                        Error::Window(
                            "no unshifted Shift key on the current keyboard layout".to_string(),
                        )
                    })?;
                fake_key(conn, KEY_PRESS_EVENT, shift_keycode)?;
                fake_key(conn, KEY_PRESS_EVENT, keycode)?;
                fake_key(conn, KEY_RELEASE_EVENT, keycode)?;
                fake_key(conn, KEY_RELEASE_EVENT, shift_keycode)?;
            } else {
                fake_key(conn, KEY_PRESS_EVENT, keycode)?;
                fake_key(conn, KEY_RELEASE_EVENT, keycode)?;
            }
        }
        thread::sleep(Duration::from_millis(INTER_KEY_DELAY_MS));
    }
    for keycode in held.iter().rev() {
        fake_key(conn, KEY_RELEASE_EVENT, *keycode)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CONTROL_L, Error, Keymap, index_keymap, parse_keys, parse_text, resolve_key, send_keys,
    };
    use crate::SendKeysParams;

    #[tokio::test]
    async fn test_usage() {
        // All rejected before any X connection or wmctrl call is attempted.
        let bad_id = send_keys(SendKeysParams {
            window_id: "nope".to_string(),
            text: None,
            keys: Some("Enter".to_string()),
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(bad_id, Err(Error::Window(_))));

        let both = send_keys(SendKeysParams {
            window_id: "0x1".to_string(),
            text: Some("hi".to_string()),
            keys: Some("Enter".to_string()),
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(both, Err(Error::Window(_))));

        let neither = send_keys(SendKeysParams {
            window_id: "0x1".to_string(),
            text: None,
            keys: None,
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(neither, Err(Error::Window(_))));

        let empty_text = send_keys(SendKeysParams {
            window_id: "0x1".to_string(),
            text: Some(String::new()),
            keys: None,
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(empty_text, Err(Error::Window(_))));
    }

    #[test]
    fn test_usage_parse() {
        let text = parse_text("A\n\t").unwrap();
        assert_eq!(text.len(), 3);
        assert_eq!(text[0].keysym, 'A' as u32);
        assert!(!text[0].hold);
        assert!(parse_text("é").is_err());

        let chord = parse_keys("Ctrl+Shift+F5").unwrap();
        assert_eq!(chord.len(), 3);
        assert!(chord[0].hold);
        assert_eq!(chord[0].keysym, CONTROL_L);
        assert!(!chord[2].hold);
        assert!(chord[2].keysym >= 0xFFBE);
        assert!(parse_keys("Ctrl+Whee").is_err());
        assert!(parse_keys("").is_err());
    }

    #[test]
    fn test_usage_resolve() {
        let mut keymap = Keymap::default();
        keymap.plain.insert(0x61, 38);
        keymap.shifted.insert(0x41, 38);
        assert_eq!(resolve_key(0x61, &keymap).unwrap(), (38, false));
        assert_eq!(resolve_key(0x41, &keymap).unwrap(), (38, true));
        assert!(resolve_key(0x3A3, &keymap).is_err());
    }

    #[test]
    fn test_usage_keymap() {
        let keymap = index_keymap(&[0x61, 0x41, 0x1B, 0], 2);
        assert_eq!(keymap.plain.get(&0x61), Some(&8));
        assert_eq!(keymap.shifted.get(&0x41), Some(&8));
        assert_eq!(keymap.plain.get(&0x1B), Some(&9));
        assert!(!keymap.shifted.contains_key(&0x1B));

        let empty = index_keymap(&[], 0);
        assert!(empty.plain.is_empty() && empty.shifted.is_empty());
    }

    /// Spawns a real xterm, finds its window, and types text plus a chord into it. No
    /// pixel-diffing (too flaky) — this only confirms the full X connect / get_keyboard_mapping
    /// / xtest_fake_input path doesn't error against a real window.
    #[tokio::test]
    async fn test_usage_live_display() {
        use crate::window_info_method;

        crate::test_support::assert_live_display();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let mut child = std::process::Command::new("xterm")
            .spawn()
            .expect("spawning xterm for live send_keys test");
        std::thread::sleep(std::time::Duration::from_millis(800));

        let windows = window_info_method::list().expect("list_windows for live send_keys test");
        let window = windows
            .iter()
            .find(|w| w.pid == child.id())
            .expect("xterm window not found by pid");

        let text_result = send_keys(SendKeysParams {
            window_id: window.id.clone(),
            text: Some("hello world".to_string()),
            keys: None,
            note: None,
            send_to_telegram: true,
        })
        .await;

        let keys_result = send_keys(SendKeysParams {
            window_id: window.id.clone(),
            text: None,
            keys: Some("Ctrl+A".to_string()),
            note: None,
            send_to_telegram: true,
        })
        .await;

        let _ = child.kill();
        let _ = child.wait();

        assert!(text_result.is_ok(), "{text_result:?}");
        assert!(keys_result.is_ok(), "{keys_result:?}");
    }
}
