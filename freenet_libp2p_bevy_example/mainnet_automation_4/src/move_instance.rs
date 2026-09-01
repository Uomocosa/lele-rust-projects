use crate::Error;
use crate::hold_key;
use crate::raise_window;

pub fn move_instance(window_id: &str, right: bool, duration_ms: u64) -> Result<(), Error> {
    raise_window::raise_window(window_id)?;
    let keysym = if right { 0x64 } else { 0x61 };
    hold_key::hold_key(keysym, duration_ms)
}

// no test_usage necessary
