use freenet_stdlib::prelude::UpdateData;

use crate::roster;

pub fn decode_roster_update(update: &UpdateData) -> Option<roster::RosterState> {
    let bytes = match update {
        UpdateData::State(state) => state.as_ref(),
        UpdateData::Delta(delta) => delta.as_ref(),
        _ => return None,
    };
    bincode::deserialize(bytes).ok()
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::prelude::{State, UpdateData};

    use crate::net_id;

    use super::decode_roster_update;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut roster = roster::RosterState::default();
        roster.insert(
            net_id::NetworkId(1),
            roster::PeerEntry {
                peer_id: "peer-1".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        let update = UpdateData::State(State::from(bincode::serialize(&roster).unwrap()));

        let decoded = decode_roster_update(&update).unwrap();
        assert_eq!(decoded, roster);
    }
}
