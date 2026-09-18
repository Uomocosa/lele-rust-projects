use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::support;

fn click_gossip(from: &str, owner: u64) -> p2p::Event<clicker::CursorMsg> {
    let data = bincode::serialize(&clicker::CursorMsg::Click {
        owner: net_id::NetworkId(owner),
        delta: 1,
    })
    .unwrap_or_default();
    p2p::Event::Gossip {
        topic: clicker::click_topic(&clicker::ActiveLobby("alpha".to_string())),
        from: from.to_string(),
        data,
    }
}

fn dump(mesh: &mut testing::Mesh, one: usize) -> String {
    let app = &mut mesh.apps[one];
    let mut query = app.world_mut().query::<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &clicker::ClickCounter,
    )>();
    let slots: Vec<String> = query
        .iter(app.world())
        .map(|(owner, player, counter)| {
            format!(
                "(owner={:?} player={:?} count={})",
                **owner,
                player.map(|number| **number),
                **counter
            )
        })
        .collect();
    let pending: Vec<String> = app
        .world()
        .resource::<clicker::PendingClicks>()
        .items
        .iter()
        .map(|item| {
            format!(
                "(sender={:?} owner={:?} delta={} abs={})",
                *item.sender, *item.owner, item.delta, item.absolute
            )
        })
        .collect();
    let stones: Vec<String> = app
        .world()
        .resource::<clicker::ScoreTombstones>()
        .iter()
        .map(|(logical, count)| format!("{logical}:{count}"))
        .collect();
    format!("slots={slots:?} pending={pending:?} stones={stones:?}")
}

fn app_one(mesh: &mut testing::Mesh) -> usize {
    mesh.apps
        .iter()
        .position(|app| *app.world().resource::<clicker::InstanceInfo>().own_id == 1)
        .unwrap_or_default()
}

#[test]
fn unlabeled_clicks_park_exact() {
    let mut mesh = testing::Mesh::three();
    let one = app_one(&mut mesh);
    for _ in 0..17 {
        support::push_to(&mut mesh, 1, click_gossip("peer-3", 3));
    }
    for _ in 0..5 {
        support::push_to(&mut mesh, 1, click_gossip("peer-2", 2));
    }
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::SyncAck {
                target: net_id::NetworkId(1),
                entries: vec![(net_id::NetworkId(2), 5), (net_id::NetworkId(3), 17)],
            },
        },
    );
    mesh.step();
    mesh.step();
    mesh.step();
    let counts = mesh.counts();
    assert_eq!(
        counts[one],
        testing::MeshCount::wanted(0, 5, 17),
        "pre-label traffic lands exact: {counts:?} {}",
        dump(&mut mesh, one),
    );
    assert!(
        mesh.apps[one]
            .world()
            .resource::<clicker::PendingClicks>()
            .items
            .is_empty(),
        "nothing left parked"
    );
    mesh.apps[one]
        .world_mut()
        .resource_mut::<roster::Roster>()
        .remove_entry("alpha", [2u8; 32]);
    mesh.apps[one]
        .world_mut()
        .resource_mut::<roster::Roster>()
        .remove_entry("alpha", [3u8; 32]);
    for _ in 0..3 {
        mesh.step();
    }
    let stones = mesh.apps[one]
        .world()
        .resource::<clicker::ScoreTombstones>();
    assert_eq!(
        stones.get(&2),
        Some(&5),
        "pruned score parks exact, no inflation"
    );
    assert_eq!(
        stones.get(&3),
        Some(&17),
        "pruned score parks exact, no inflation"
    );
    let counts = mesh.counts();
    assert_eq!(
        counts[one].global, 22,
        "retained total stays exact after prune: {counts:?}"
    );
}
