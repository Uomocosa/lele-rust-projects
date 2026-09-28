use std::time::{Duration, Instant};

use serde_json::json;

use crate::Error;
use crate::cdp;

pub fn navigate(session: &mut cdp::Session, url: &str) -> Result<(), Error> {
    cdp::call(session, "Page.navigate", &json!({ "url": url }))?;
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(20))
        .ok_or_else(|| Error::Timeout(url.to_string()))?;
    loop {
        std::thread::sleep(Duration::from_millis(50));
        let state = cdp::evaluate(session, "document.readyState")?;
        if state.as_str() == Some("complete") {
            break;
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout(format!("page load {url}")));
        }
    }
    cdp::evaluate(session, "document.fonts.ready.then(() => true)")?;
    Ok(())
}

// no test_usage necessary
