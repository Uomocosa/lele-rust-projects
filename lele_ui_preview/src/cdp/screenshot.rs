use base64::Engine;
use serde_json::{Value, json};

use crate::Error;
use crate::cdp;

pub fn screenshot(session: &mut cdp::Session) -> Result<Vec<u8>, Error> {
    let result = cdp::call(
        session,
        "Page.captureScreenshot",
        &json!({ "format": "png" }),
    )?;
    let data = result
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Cdp("captureScreenshot returned no data".to_string()))?;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| Error::Cdp(format!("screenshot base64: {e}")))
}

// no test_usage necessary
