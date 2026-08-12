use std::{collections::HashMap, thread, time::Duration};

use rmcp::model::{CallToolResult, ContentBlock};
use x11rb::{
    connection::Connection as _,
    protocol::{xproto::ConnectionExt as _, xtest::ConnectionExt as _},
    rust_connection::RustConnection,
};

use crate::{Error, KeyboardInput, KeyboardKey, SendKeysParams, window_info_method};

const KEY_PRESS_EVENT: u8 = 2;
const KEY_RELEASE_EVENT: u8 = 3;
const FIRST_KEYCODE: u8 = 8;
const KEYCODE_COUNT: u8 = 248;
const INTER_KEY_DELAY_MS: u64 = 10;
const MAX_UNITS: usize = 5000;
const MAX_WAIT_MS: u64 = 120_000;

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
enum Step {
    Tap(u32),
    Hold { keysym: u32, duration_ms: u64 },
    Chord(Vec<u32>),
    Delay(u64),
    Char(u32),
}

#[derive(Debug, Default)]
struct Keymap {
    plain: HashMap<u32, u8>,
    shifted: HashMap<u32, u8>,
}

/// Owns the keycodes that are currently pressed but not yet released. Releasing happens on the
/// happy path via `release_all`, and on error via `Drop` — so a failed send can never leave a
/// modifier (Ctrl/Shift/Alt) physically stuck down.
struct HeldKeys<'a> {
    conn: &'a RustConnection,
    keycodes: Vec<u8>,
}

impl<'a> HeldKeys<'a> {
    fn new(conn: &'a RustConnection) -> Self {
        Self {
            conn,
            keycodes: Vec::new(),
        }
    }

    fn press(&mut self, keycode: u8) -> Result<(), Error> {
        fake_key(self.conn, KEY_PRESS_EVENT, keycode)?;
        self.keycodes.push(keycode);
        Ok(())
    }

    fn release_all(&mut self) -> Result<(), Error> {
        let mut first_err = None;
        for &keycode in self.keycodes.iter().rev() {
            if let Err(e) = fake_key(self.conn, KEY_RELEASE_EVENT, keycode)
                && first_err.is_none()
            {
                first_err = Some(e);
            }
        }
        self.keycodes.clear();
        if let Some(e) = first_err {
            return Err(e);
        }
        Ok(())
    }
}

impl Drop for HeldKeys<'_> {
    fn drop(&mut self) {
        let _ = self.release_all();
    }
}

pub async fn send_keys(params: SendKeysParams) -> Result<CallToolResult, Error> {
    let window_id = &params.window_id;
    if !window_info_method::is_valid_id(window_id) {
        return Err(Error::Window(format!(
            "invalid window_id {window_id:?}: expected hex like \"0x03a00004\" (see list_windows)"
        )));
    }
    let steps = build_steps(&params.inputs)?;
    enforce_cap(&steps)?;

    // XTEST synthesizes input at the *root*, so it lands on whatever is topmost. Raise the
    // target first or an overlapping window eats the keystrokes.
    super::raise_window::raise_window(window_id);

    let (conn, _screen) =
        x11rb::connect(None).map_err(|e| Error::Window(format!("connecting to X display: {e}")))?;
    let keymap = keymap(&conn)?;
    execute(&conn, &steps, &keymap)?;
    conn.flush()
        .map_err(|e| Error::Window(format!("flushing keystrokes: {e}")))?;

    let summary = super::summarize_inputs(&params.inputs);
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "{summary} in {window_id}; screenshot the window to confirm it took effect"
    ))]))
}

// needed helper:
fn build_steps(inputs: &[KeyboardInput]) -> Result<Vec<Step>, Error> {
    if inputs.is_empty() {
        return Err(Error::Window(
            "nothing to send: provide a non-empty inputs sequence".to_string(),
        ));
    }
    let mut steps = Vec::new();
    for input in inputs {
        match input {
            KeyboardInput::Tap { key } => steps.push(Step::Tap(keysym(key)?)),
            KeyboardInput::Hold { key, duration_ms } => steps.push(Step::Hold {
                keysym: keysym(key)?,
                duration_ms: *duration_ms,
            }),
            KeyboardInput::Chord { keys } => {
                if keys.is_empty() {
                    return Err(Error::Window(
                        "chord must contain at least one key".to_string(),
                    ));
                }
                let mut resolved = Vec::with_capacity(keys.len());
                for key in keys {
                    resolved.push(keysym(key)?);
                }
                steps.push(Step::Chord(resolved));
            }
            KeyboardInput::Delay { duration_ms } => steps.push(Step::Delay(*duration_ms)),
            KeyboardInput::Text { text } => {
                for c in text.chars() {
                    steps.push(Step::Char(text_char_keysym(c)?));
                }
            }
        }
    }
    Ok(steps)
}

// needed helper:
fn enforce_cap(steps: &[Step]) -> Result<(), Error> {
    let mut units = 0usize;
    let mut wait_ms = 0u64;
    for step in steps {
        match step {
            Step::Tap(_) | Step::Delay(_) => units += 1,
            Step::Hold { duration_ms, .. } => {
                units += 1;
                wait_ms += duration_ms;
            }
            Step::Chord(keys) => units += keys.len(),
            Step::Char(_) => units += 1,
        }
    }
    if units > MAX_UNITS {
        return Err(Error::Window(format!(
            "input plan is too large ({units} units, cap {MAX_UNITS}): make it more deliberate"
        )));
    }
    if wait_ms > MAX_WAIT_MS {
        return Err(Error::Window(format!(
            "total hold/delay time is too long ({wait_ms}ms, cap {MAX_WAIT_MS}ms)"
        )));
    }
    Ok(())
}

// needed helper:
fn keysym(key: &KeyboardKey) -> Result<u32, Error> {
    named_keysym(key.as_str()).ok_or_else(|| {
        Error::Window(format!(
            "unknown key {key:?}: use a modifier (Ctrl, Shift, Alt, Super, Meta), a named key \
             (Enter, Tab, BackSpace, Escape, Delete, Insert, Home, End, PageUp, PageDown, arrows, \
             Space, F1-F12) or a single printable ASCII character"
        ))
    })
}

// needed helper:
fn text_char_keysym(c: char) -> Result<u32, Error> {
    match c {
        '\n' => Ok(RETURN),
        '\t' => Ok(TAB),
        ' '..='~' => Ok(c as u32),
        _ => Err(Error::Window(format!(
            "unsupported character {c:?}: text is limited to printable ASCII plus \\n and \\t"
        ))),
    }
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
    let lower = name.to_ascii_lowercase();
    if let Some(n) = lower
        .strip_prefix('f')
        .and_then(|digits| digits.parse::<u32>().ok())
        .filter(|&n| (1..=12).contains(&n))
    {
        return Some(F1 + n - 1);
    }
    match lower.as_str() {
        "ctrl" | "control" => Some(CONTROL_L),
        "shift" => Some(SHIFT_L),
        "alt" => Some(ALT_L),
        "super" | "win" => Some(SUPER_L),
        "meta" => Some(META_L),
        "enter" | "return" => Some(RETURN),
        "tab" => Some(TAB),
        "backspace" => Some(BACK_SPACE),
        "escape" | "esc" => Some(ESCAPE),
        "delete" | "del" => Some(DELETE),
        "insert" | "ins" => Some(INSERT),
        "home" => Some(HOME),
        "end" => Some(END),
        "pageup" => Some(PAGE_UP),
        "pagedown" => Some(PAGE_DOWN),
        "left" => Some(LEFT),
        "right" => Some(RIGHT),
        "up" => Some(UP),
        "down" => Some(DOWN),
        "space" => Some(SPACE),
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
fn execute(conn: &RustConnection, steps: &[Step], keymap: &Keymap) -> Result<(), Error> {
    let mut held = HeldKeys::new(conn);
    for step in steps {
        match step {
            Step::Tap(keysym) => {
                tap_key(conn, *keysym, keymap)?;
            }
            Step::Hold {
                keysym,
                duration_ms,
            } => {
                let (keycode, needs_shift) = resolve_key(*keysym, keymap)?;
                if needs_shift {
                    return Err(Error::Window(format!(
                        "keysym {keysym:#x} is only reachable via Shift; cannot hold it"
                    )));
                }
                held.press(keycode)?;
                thread::sleep(Duration::from_millis(*duration_ms));
                held.release_all()?;
            }
            Step::Chord(keysyms) => {
                for keysym in keysyms {
                    let (keycode, needs_shift) = resolve_key(*keysym, keymap)?;
                    if needs_shift {
                        return Err(Error::Window(format!(
                            "keysym {keysym:#x} is only reachable via Shift; cannot hold it in a \
                             chord"
                        )));
                    }
                    held.press(keycode)?;
                }
                held.release_all()?;
            }
            Step::Delay(duration_ms) => {
                thread::sleep(Duration::from_millis(*duration_ms));
            }
            Step::Char(keysym) => {
                tap_key(conn, *keysym, keymap)?;
            }
        }
        thread::sleep(Duration::from_millis(INTER_KEY_DELAY_MS));
    }
    held.release_all()?;
    Ok(())
}

// needed helper:
fn tap_key(conn: &RustConnection, keysym: u32, keymap: &Keymap) -> Result<(), Error> {
    let (keycode, needs_shift) = resolve_key(keysym, keymap)?;
    if needs_shift {
        let shift_keycode = keymap
            .plain
            .get(&SHIFT_L)
            .or_else(|| keymap.plain.get(&SHIFT_R))
            .copied()
            .ok_or_else(|| {
                Error::Window("no unshifted Shift key on the current keyboard layout".to_string())
            })?;
        fake_key(conn, KEY_PRESS_EVENT, shift_keycode)?;
        fake_key(conn, KEY_PRESS_EVENT, keycode)?;
        fake_key(conn, KEY_RELEASE_EVENT, keycode)?;
        fake_key(conn, KEY_RELEASE_EVENT, shift_keycode)?;
    } else {
        fake_key(conn, KEY_PRESS_EVENT, keycode)?;
        fake_key(conn, KEY_RELEASE_EVENT, keycode)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CONTROL_L, Error, Keymap, MAX_UNITS, MAX_WAIT_MS, Step, build_steps, enforce_cap,
        index_keymap, named_keysym, resolve_key, send_keys,
    };
    use crate::{KeyboardInput, KeyboardKey, SendKeysParams, test_support};

    #[tokio::test]
    async fn test_usage() {
        let bad_id = send_keys(SendKeysParams {
            window_id: "nope".to_string(),
            inputs: vec![KeyboardInput::Chord {
                keys: vec![
                    KeyboardKey("ctrl".to_string()),
                    KeyboardKey("a".to_string()),
                ],
            }],
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(bad_id, Err(Error::Window(_))));

        let empty = send_keys(SendKeysParams {
            window_id: "0x1".to_string(),
            inputs: Vec::new(),
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(empty, Err(Error::Window(_))));
    }

    #[test]
    fn test_usage_build_steps() {
        let inputs = vec![
            KeyboardInput::Tap {
                key: KeyboardKey("a".to_string()),
            },
            KeyboardInput::Text {
                text: "A\n".to_string(),
            },
            KeyboardInput::Chord {
                keys: vec![
                    KeyboardKey("ctrl".to_string()),
                    KeyboardKey("a".to_string()),
                ],
            },
        ];
        let steps = build_steps(&inputs).unwrap();
        assert!(matches!(steps[0], Step::Tap(0x61)));
        assert!(matches!(steps[1], Step::Char(0x41)));
        assert!(matches!(steps[2], Step::Char(0xFF0D)));
        assert!(matches!(steps[3], Step::Chord(_)));

        assert!(build_steps(&[]).is_err());
        let bad = KeyboardInput::Chord {
            keys: vec![KeyboardKey("nope".to_string())],
        };
        assert!(build_steps(&[bad]).is_err());
    }

    #[test]
    fn test_usage_cap() {
        let text = "d".repeat(MAX_UNITS + 1);
        let steps = build_steps(&[KeyboardInput::Text { text }]).unwrap();
        assert!(enforce_cap(&steps).is_err());

        let too_long = build_steps(&[KeyboardInput::Hold {
            key: KeyboardKey("a".to_string()),
            duration_ms: MAX_WAIT_MS + 1,
        }])
        .unwrap();
        assert!(enforce_cap(&too_long).is_err());
    }

    #[test]
    fn test_usage_named() {
        assert_eq!(named_keysym("Ctrl"), Some(CONTROL_L));
        assert_eq!(named_keysym("F5"), Some(0xFFC2));
        assert_eq!(named_keysym("d"), Some(0x64));
        assert_eq!(named_keysym("Whee"), None);
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

    #[test]
    fn test_usage_summarize() {
        let inputs = vec![KeyboardInput::Tap {
            key: KeyboardKey("a".to_string()),
        }];
        assert!(super::super::summarize_inputs(&inputs).contains("tap a"));
        assert!(super::super::summarize_inputs(&[]).contains("sent keys"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_usage_live_display() {
        use crate::window_info_method;

        test_support::assert_live_display();
        let _guard = test_support::live_test_lock().lock().await;

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
            inputs: vec![KeyboardInput::Text {
                text: "hello world".to_string(),
            }],
            note: None,
            send_to_telegram: true,
        })
        .await;

        let chord_result = send_keys(SendKeysParams {
            window_id: window.id.clone(),
            inputs: vec![KeyboardInput::Chord {
                keys: vec![
                    KeyboardKey("ctrl".to_string()),
                    KeyboardKey("a".to_string()),
                ],
            }],
            note: None,
            send_to_telegram: true,
        })
        .await;

        let _ = child.kill();
        let _ = child.wait();

        assert!(text_result.is_ok(), "{text_result:?}");
        assert!(chord_result.is_ok(), "{chord_result:?}");
    }
}
