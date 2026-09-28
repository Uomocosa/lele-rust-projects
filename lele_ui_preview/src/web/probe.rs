use serde_json::Value;

use crate::Error;
use crate::cdp;
use crate::web;

pub fn probe(session: &mut cdp::Session) -> Result<web::Probe, Error> {
    let selector = serde_json::to_string(web::INTERACTIVE_SELECTOR)?;
    let script = web::PROBE_JS.replace("__SELECTOR__", &selector);
    match cdp::evaluate(session, &script)? {
        Value::String(text) => Ok(serde_json::from_str(&text)?),
        other => Err(Error::Cdp(format!("probe returned {other}"))),
    }
}

// no test_usage necessary
