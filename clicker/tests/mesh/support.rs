use clicker_lib::clicker;
use freenet_libp2p_bevy_plugin::{net_id, p2p};

pub fn push_to(
    mesh: &mut clicker_lib::testing::Mesh,
    owner: u64,
    event: p2p::Event<clicker::CursorMsg>,
) -> bool {
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == owner {
            app.world_mut()
                .resource_mut::<p2p::Events<clicker::CursorMsg>>()
                .push(event);
            return true;
        }
    }
    false
}

pub fn move_gossip(from: &str, owner: u64) -> p2p::Event<clicker::CursorMsg> {
    let msg = clicker::CursorMsg::Move {
        owner: net_id::NetworkId(owner),
        pos: [30.0, 40.0],
    };
    let data = bincode::serialize(&msg).unwrap_or_default();
    p2p::Event::Gossip {
        topic: clicker::pos_topic(&clicker::ActiveLobby("alpha".to_string())),
        from: from.to_string(),
        data,
    }
}

pub fn snapshot_chunk(entries: Vec<(u64, i32)>, global: i32) -> p2p::Event<clicker::CursorMsg> {
    let snapshot = clicker::Snapshot {
        entries: entries
            .into_iter()
            .map(|(id, count)| (net_id::NetworkId(id), count))
            .collect(),
        global,
    };
    p2p::Event::HistoryChunk {
        lobby: "alpha".to_string(),
        chunk: clicker::SNAPSHOT_CHUNK,
        data: clicker::encode_snapshot(&snapshot),
    }
}
