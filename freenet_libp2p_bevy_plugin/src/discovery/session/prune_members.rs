use crate::discovery;
use discovery::id::now_epoch;
use discovery::session::Session;

pub fn prune_members(session: &mut Session, ttl_secs: u64) {
    let now = now_epoch();
    if let Some(room) = session.room.as_mut() {
        room.members
            .retain(|_, member| now.saturating_sub(*member.presence.updated_at) <= ttl_secs);
    }
}

// no test_usage necessary
