use std::time::{Duration, Instant};

use crate::discovery;
use discovery::id::now_epoch;
use discovery::link::{NetLink, dialable};
use discovery::session::prune_members::prune_members;
use discovery::session::{Session, broadcast_hello, dial_candidates};
use discovery::{Event, Timing};

pub fn maintain(
    session: &mut Session,
    link: &mut NetLink,
    timing: &Timing,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    refresh_observed(session, link);
    let now = Instant::now();
    dial_candidates(session, link, Duration::from_secs(timing.redial_secs), now);
    let hello_due = session.last_hello.is_none_or(|last| {
        now.saturating_duration_since(last) >= Duration::from_secs(timing.hello_secs)
    });
    if hello_due && session.room.is_some() {
        broadcast_hello(session, link);
        session.last_hello = Some(now);
    }
    prune_members(
        session,
        now_epoch(),
        timing.member_grace_secs,
        timing.presence_ttl_secs,
        events,
    );
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
