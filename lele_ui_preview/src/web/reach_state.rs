use crate::Error;
use crate::cdp;
use crate::web;

pub fn reach_state(
    session: &mut cdp::Session,
    target: &web::Target,
    state: &web::WebState,
) -> Result<(), Error> {
    cdp::navigate(session, &format!("{}{}", target.base_url, state.location))?;
    for step in &state.steps {
        web::apply_action(session, &target.base_url, step, target.settle_ms)?;
    }
    Ok(())
}

// no test_usage necessary
