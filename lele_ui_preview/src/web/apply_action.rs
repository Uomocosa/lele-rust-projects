use std::time::Duration;

use serde_json::Value;

use crate::Error;
use crate::cdp;
use crate::web;

pub fn apply_action(
    session: &mut cdp::Session,
    base_url: &str,
    action: &web::Action,
    settle_ms: u64,
) -> Result<(), Error> {
    match action {
        web::Action::Navigate { location, .. } => {
            cdp::navigate(session, &format!("{base_url}{location}"))?;
        }
        web::Action::Click { index, .. } => {
            run(session, *index, "el.click();")?;
        }
        web::Action::Fill { index, text, .. } => {
            let text = serde_json::to_string(text)?;
            let body = format!(
                "el.focus(); el.value = {text}; el.dispatchEvent(new Event('input', {{ bubbles: true }}));"
            );
            run(session, *index, &body)?;
        }
        web::Action::Key { key } => cdp::press_key(session, key)?,
        web::Action::ScrollBottom => {
            cdp::evaluate(
                session,
                "window.scrollTo(0, document.documentElement.scrollHeight); true",
            )?;
        }
    }
    std::thread::sleep(Duration::from_millis(settle_ms));
    Ok(())
}

// needed helper: run a snippet against the n-th interactive element of the page
fn run(session: &mut cdp::Session, index: usize, body: &str) -> Result<(), Error> {
    let selector = serde_json::to_string(web::INTERACTIVE_SELECTOR)?;
    let script = format!(
        "(() => {{ const el = document.querySelectorAll({selector})[{index}]; \
         if (!el) return false; {body} return true; }})()"
    );
    if cdp::evaluate(session, &script)? == Value::Bool(true) {
        Ok(())
    } else {
        Err(Error::Cdp(format!(
            "interactive element #{index} not found"
        )))
    }
}

// no test_usage necessary
