use super::super::link::{NetLink, dialable};
use super::super::timing::Timing;
use super::announce::announce;
use super::dial_known::dial_known;
use super::prune_members::prune_members;
use super::session::Session;

pub fn maintain(session: &mut Session, link: &mut NetLink, timing: &Timing) {
    refresh_observed(session, link);
    dial_known(session, link);
    announce(session, link);
    prune_members(session, timing.presence_ttl_secs);
}

// needed helper: adopts libp2p-observed addresses so peers can dial us back
fn refresh_observed(session: &mut Session, link: &mut NetLink) {
    if !link.observed.has_changed().unwrap_or(false) {
        return;
    }
    let observed = link.observed.borrow_and_update().clone();
    if let Some(addrs) = observed {
        session.addrs = dialable(addrs);
    }
}

// no test_usage necessary
