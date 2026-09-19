use std::any::type_name;

use bevy::prelude::Name;
use bevy::ui::UiGlobalTransform;
use serde_json::Value;

use super::named_button::NamedButton;
use super::parse_named_buttons;
use super::request;

/// # Errors
/// Returns an error when the app cannot be reached over the Bevy Remote Protocol.
pub fn named_buttons(port: u16) -> Result<Vec<NamedButton>, String> {
    let params = serde_json::json!({
        "data": {
            "components": [type_name::<Name>(), type_name::<UiGlobalTransform>()],
            "option": [],
            "has": [],
        },
        "filter": { "with": [], "without": [] },
        "strict": false,
    });
    let result: Value = request::request(port, "world.query", &params)?;
    Ok(parse_named_buttons::parse_named_buttons(&result))
}

// no test_usage necessary
