use std::time::Duration;

use super::redial_once::redial_once;
use super::up_link::UpLink;

pub fn redial_missing(link: &mut UpLink, name: &str, now: Duration) {
    let targets = link.redial_targets.clone();
    let every = link.redial_every;
    for target in targets {
        if link.outbound.contains_key(target) || link.dead.contains(target) {
            continue;
        }
        let prev = link
            .last_redial
            .get(target)
            .copied()
            .unwrap_or(Duration::ZERO);
        if now.saturating_sub(prev) >= every {
            link.last_redial.insert(target.to_string(), now);
            tokio::spawn(redial_once(
                target,
                every,
                name.to_string(),
                link.accept_tx.clone(),
            ));
        }
    }
}

// no test_usage necessary
