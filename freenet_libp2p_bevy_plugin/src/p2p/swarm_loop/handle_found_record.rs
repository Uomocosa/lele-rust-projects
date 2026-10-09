use libp2p::kad;

use crate::net_id;
use crate::p2p;

pub fn handle_found_record<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_record: kad::PeerRecord,
) {
    let record = peer_record.record;
    let key = String::from_utf8_lossy(record.key.as_ref());
    let Some((room, chunk)) = p2p::parse_history_key(&key) else {
        return;
    };
    event_tx
        .send(p2p::Event::HistoryChunk {
            room: net_id::RoomName(room),
            chunk,
            data: record.value,
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use super::handle_found_record;
    use crate::p2p;

    fn peer_record(key: kad::RecordKey) -> kad::PeerRecord {
        kad::PeerRecord {
            peer: None,
            record: kad::Record {
                key,
                value: vec![1, 2, 3],
                publisher: None,
                expires: None,
            },
        }
    }

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<u32>>();
        handle_found_record(&tx, peer_record(p2p::history_key("room-a", 5)));
        let Ok(p2p::Event::HistoryChunk { room, chunk, data }) = rx.try_recv() else {
            panic!("expected a history chunk");
        };
        assert_eq!((room.as_str(), chunk, data), ("room-a", 5, vec![1, 2, 3]));
    }

    #[test]
    fn test_usage_skips_malformed_keys() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<u32>>();
        handle_found_record(&tx, peer_record(kad::RecordKey::new(&"lobby/history/r/xx")));
        handle_found_record(&tx, peer_record(p2p::provider_key("room-a")));
        assert!(rx.try_recv().is_err());
    }
}
