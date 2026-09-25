use crate::discovery;
use discovery::Timing;
use discovery::link::{NetLink, dialable};
use discovery::session::Session;
use discovery::session::announce::announce;
use discovery::session::dial_known::dial_known;
use discovery::session::prune_members::prune_members;
use discovery::session::seed_from_board::seed_from_board;

pub fn maintain(session: &mut Session, link: &mut NetLink, timing: &Timing) {
    refresh_observed(session, link);
    seed_from_board(session);
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
