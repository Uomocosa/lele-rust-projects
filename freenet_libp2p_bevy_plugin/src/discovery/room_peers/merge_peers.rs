use std::collections::BTreeMap;

use crate::discovery;
use discovery::room_peers::{DiscoveryStatus, Member, Members};

pub fn merge_peers(
    members: &mut Members,
    peers: BTreeMap<discovery::id::RemotePeerId, Vec<String>>,
    now: discovery::id::EpochSecs,
    me: &discovery::id::RemotePeerId,
) {
    for (peer, addrs) in peers {
        if peer == *me || peer.is_empty() {
            continue;
        }
        match members.get_mut(&peer) {
            Some(member) => {
                if *member.presence.updated_at < *now {
                    member.presence.addrs = addrs;
                    member.presence.updated_at = now;
                }
            }
            None => {
                members.insert(
                    peer,
                    Member {
                        presence: discovery::id::Presence {
                            addrs,
                            updated_at: now,
                        },
                        status: DiscoveryStatus::Known,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::merge_peers;
    use crate::discovery;
    use crate::discovery::room_peers::Members;

    #[test]
    fn test_usage() {
        let me = discovery::id::RemotePeerId("me".to_string());
        let mut members = Members::new();
        let mut peers = BTreeMap::new();
        peers.insert(
            discovery::id::RemotePeerId("peer".to_string()),
            vec!["/ip4/127.0.0.1/tcp/1".to_string()],
        );
        peers.insert(
            discovery::id::RemotePeerId("me".to_string()),
            vec!["/ip4/127.0.0.1/tcp/2".to_string()],
        );
        merge_peers(&mut members, peers, discovery::id::EpochSecs(5), &me);
        assert_eq!(members.len(), 1);
        assert!(members.contains_key(&discovery::id::RemotePeerId("peer".to_string())));
    }
}
