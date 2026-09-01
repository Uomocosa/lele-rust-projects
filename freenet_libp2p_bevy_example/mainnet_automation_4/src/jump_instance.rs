use crate::Error;
use crate::hold_key;
use crate::raise_window;

pub fn jump_instance(window_id: &str, duration_ms: u64) -> Result<(), Error> {
    raise_window::raise_window(window_id)?;
    hold_key::hold_key(0x20, duration_ms)
}

// no test_usage necessary — needs a live X display; exercised via the integration run
