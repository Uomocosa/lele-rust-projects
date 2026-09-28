use serde_json::json;

use crate::Error;
use crate::cdp;
use crate::config;

pub fn set_viewport(session: &mut cdp::Session, viewport: &config::Viewport) -> Result<(), Error> {
    cdp::call(
        session,
        "Emulation.setDeviceMetricsOverride",
        &json!({
            "width": viewport.width,
            "height": viewport.height,
            "deviceScaleFactor": 1,
            "mobile": viewport.width < 700,
        }),
    )?;
    Ok(())
}

// no test_usage necessary
