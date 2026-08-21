use std::collections::HashMap;
use std::time::Duration;

use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

use crate::Error;

const KEY_PRESS_EVENT: u8 = 2;
const KEY_RELEASE_EVENT: u8 = 3;
const FIRST_KEYCODE: u8 = 8;
const KEYCODE_COUNT: u8 = 248;

pub fn hold_key(keysym: u32, duration_ms: u64) -> Result<(), Error> {
    let (conn, _) = connect()?;
    let keycode = keycode_for(keysym, &keymap(&conn)?)?;
    fake_key(&conn, KEY_PRESS_EVENT, keycode)?;
    std::thread::sleep(Duration::from_millis(duration_ms));
    fake_key(&conn, KEY_RELEASE_EVENT, keycode)?;
    Ok(())
}

// needed helper:
fn connect() -> Result<(RustConnection, usize), Error> {
    x11rb::connect(None).map_err(|e| Error::X11(format!("connecting to X display: {e}")))
}

// needed helper:
fn keymap(conn: &RustConnection) -> Result<HashMap<u32, u8>, Error> {
    let reply = conn
        .get_keyboard_mapping(FIRST_KEYCODE, KEYCODE_COUNT)
        .map_err(|e| Error::X11(format!("get_keyboard_mapping: {e}")))?
        .reply()
        .map_err(|e| Error::X11(format!("get_keyboard_mapping reply: {e}")))?;
    let per = reply.keysyms_per_keycode as usize;
    let mut map = HashMap::new();
    if per == 0 {
        return Ok(map);
    }
    for (i, &keysym) in reply.keysyms.iter().enumerate() {
        if keysym == 0 {
            continue;
        }
        let keycode = FIRST_KEYCODE + (i / per) as u8;
        if i % per == 0 {
            map.entry(keysym).or_insert(keycode);
        }
    }
    Ok(map)
}

// needed helper:
fn keycode_for(keysym: u32, map: &HashMap<u32, u8>) -> Result<u8, Error> {
    map.get(&keysym)
        .copied()
        .ok_or_else(|| Error::Drive(format!("keysym {keysym:#x} not on the current layout")))
}

// needed helper:
fn fake_key(conn: &RustConnection, event: u8, keycode: u8) -> Result<(), Error> {
    conn.xtest_fake_input(event, keycode, 0, 0, 0, 0, 0)
        .map_err(|e| Error::X11(format!("xtest_fake_input: {e}")))?;
    conn.flush()
        .map_err(|e| Error::X11(format!("flushing keystroke: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::KEY_PRESS_EVENT;

    #[test]
    fn test_usage() {
        assert_eq!(KEY_PRESS_EVENT, 2);
    }
}
