use super::center_of;
use super::click_xy;
use super::named_buttons;

/// # Errors
/// Returns an error when the button is absent, the app cannot be reached, or
/// the click cannot be sent.
pub fn click_button(title: &str, port: u16, needle: &str) -> Result<(), String> {
    let buttons = named_buttons::named_buttons(port)?;
    let (x, y) = center_of::center_of(&buttons, needle)
        .ok_or_else(|| format!("brp: no button matching {needle}"))?;
    click_xy::click_xy(title, x, y)
}

// no test_usage necessary
