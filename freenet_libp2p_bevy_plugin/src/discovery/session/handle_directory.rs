use crate::discovery;
use discovery::session::{Output, Session};

pub fn handle_directory(
    session: &mut Session,
    directory: discovery::Directory,
    now: discovery::EpochSecs,
) {
    let ttl_secs = session.timing.presence_ttl_secs;
    let directory = discovery::session::live_directory(directory, now, ttl_secs);
    if directory == session.directory {
        return;
    }
    session.directory = directory;
    session
        .outputs
        .push(Output::Event(discovery::Event::DirectoryChanged));
    discovery::session::seed_candidates(session);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::handle_directory;
    use crate::discovery;
    use discovery::session::{Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        session.room = Some(discovery::Room {
            name: discovery::RoomName("r".to_string()),
            members: discovery::Members::new(),
        });
        let mut members = BTreeMap::new();
        for peer in ["me", "a"] {
            members.insert(
                discovery::PeerId(peer.to_string()),
                discovery::Presence {
                    addrs: vec!["/ip4/1".to_string()],
                    updated_at: discovery::EpochSecs(3),
                },
            );
        }
        let mut directory = discovery::Directory::new();
        directory.insert(
            discovery::RoomName("r".to_string()),
            discovery::RoomRecord {
                capacity: 8,
                members,
            },
        );
        handle_directory(&mut session, directory.clone(), discovery::EpochSecs(10));
        assert_eq!(session.candidates.len(), 1);
        assert_eq!(
            std::mem::take(&mut session.outputs),
            vec![Output::Event(discovery::Event::DirectoryChanged)]
        );
        handle_directory(&mut session, directory, discovery::EpochSecs(10));
        assert_eq!(std::mem::take(&mut session.outputs), Vec::new());
    }
}
