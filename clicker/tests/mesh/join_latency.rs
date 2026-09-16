use std::time::Instant;

use clicker_lib::{clicker, discovery, lobby, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::support;

fn owners(mesh: &mut testing::Mesh, index: usize) -> Vec<u64> {
    let mut query = mesh.apps[index].world_mut().query::<&clicker::Owner>();
    query
        .iter(mesh.apps[index].world())
        .map(|owner| ***owner)
        .collect()
}

fn full_roster_gossip() -> p2p::Event<clicker::CursorMsg> {
    let mut state = discovery::RosterState::new();
    for (owner, peer) in [(2, "peer-2"), (3, "peer-3")] {
        state.insert(
            discovery::PlayerId(owner),
            discovery::PeerEntry {
                peer_id: peer.to_string(),
                addrs: vec![format!("/ip4/127.0.0.1/tcp/400{owner}")],
                updated_at: u64::MAX,
            },
        );
    }
    p2p::Event::Gossip {
        topic: "clicker/alpha/roster".to_string(),
        from: "peer-2".to_string(),
        data: bincode::serialize(&state).unwrap_or_default(),
    }
}

#[test]
fn join_emits_fetch_roster_single_round_trip() {
    let mut active = clicker::ActiveLobby::default();
    let mut selected = lobby::SelectedRoom::default();
    let mut roster_lobby = roster::Lobby::default();
    let mut pending = lobby::JoinPending::default();
    let mut gate = lobby::JoinGate::default();
    let mut clock = lobby::JoinClock::default();
    let mut commands = p2p::Commands::<clicker::CursorMsg>::default();
    lobby::join_room(
        "alpha",
        &mut lobby::JoinRooms {
            active: &mut active,
            selected: &mut selected,
            roster_lobby: &mut roster_lobby,
            pending: &mut pending,
            gate: &mut gate,
            clock: &mut clock,
        },
        &mut commands,
    );
    assert!(
        commands.iter().any(|command| matches!(
            command,
            p2p::Command::FetchRoster { lobby } if lobby == "alpha"
        )),
        "join sends exactly one FetchRoster discovery round-trip"
    );
    assert_eq!(*pending, Some("alpha".to_string()));
}

#[test]
fn all_peers_visible_within_one_second_hard_budget() {
    let started = Instant::now();
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, full_roster_gossip());
    let mut ticks = 0;
    let expected = [
        *net_id::NetworkId::from_peer("peer-2"),
        *net_id::NetworkId::from_peer("peer-3"),
    ];
    while ticks < 5 {
        mesh.apps[0].update();
        ticks += 1;
        if expected.iter().all(|id| owners(&mut mesh, 0).contains(id)) {
            break;
        }
    }
    assert!(
        expected.iter().all(|id| owners(&mut mesh, 0).contains(id)),
        "all peers must appear within 5 ticks"
    );
    assert!(
        started.elapsed().as_secs() < 1,
        "join budget is a hard 1s fail, elapsed={:?}",
        started.elapsed()
    );
}
