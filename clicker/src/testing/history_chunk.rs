use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;
use crate::constants;

#[must_use]
pub fn history_chunk(entries: Vec<(u64, i32)>, global: i32) -> p2p::Event<clicker::CursorMsg> {
    let snapshot = clicker::Snapshot {
        entries: entries
            .into_iter()
            .map(|(id, count)| (net_id::NetworkId(id), count))
            .collect(),
        global,
    };
    p2p::Event::HistoryChunk {
        lobby: "alpha".to_string(),
        chunk: constants::SNAPSHOT_CHUNK,
        data: clicker::encode_snapshot(&snapshot),
    }
}

#[cfg(test)]
mod tests {
    use super::history_chunk;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        assert!(matches!(
            history_chunk(vec![(1, 6)], 6),
            p2p::Event::HistoryChunk { .. }
        ));
    }
}
