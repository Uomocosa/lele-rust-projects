use clicker_lib::discovery;
use clicker_lib::testing;

fn directory_with(room: &str, updated_at: u64) -> discovery::DirectoryState {
    let mut state = discovery::DirectoryState::new();
    state.insert(
        room.to_string(),
        discovery::DirectoryEntry {
            params: vec![1, 2, 3],
            peer_id: format!("peer-{room}"),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{updated_at}")],
            updated_at,
        },
    );
    state
}

#[test]
fn cold_start_partitions_heal_through_directory_merge() {
    let singleton_a = directory_with("room-20250101-120000", 100);
    let singleton_b = directory_with("room-20250101-120000", 100);
    let merged = discovery::merge_directory(singleton_a, singleton_b);
    assert_eq!(merged.len(), 1);
    assert_eq!(
        discovery::pick_room(&merged, 50).map(|(room, _)| room),
        Some("room-20250101-120000".to_string())
    );
}

#[test]
fn disjoint_singletons_union_on_bridge() {
    let mut first = directory_with("room-20250101-120000", 100);
    let second = directory_with("room-20250101-120001", 110);
    first = discovery::merge_directory(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(
        discovery::pick_room(&first, 50).map(|(room, _)| room),
        Some("room-20250101-120001".to_string())
    );
}

#[test]
fn stale_rooms_never_win_pick() {
    let state = directory_with("room-20250101-120000", 100);
    assert!(discovery::pick_room(&state, 100).is_none());
    assert!(discovery::pick_room(&state, 500).is_none());
}

#[test]
fn simultaneous_learn_triggers_single_dialer() {
    let lower = discovery::decide_dial("peer-1", "peer-2", None);
    let higher = discovery::decide_dial("peer-2", "peer-1", None);
    assert_eq!(lower, discovery::DialDecision::Dial);
    assert_eq!(higher, discovery::DialDecision::Wait);
}

#[test]
fn higher_dialer_force_dials_after_two_silent_periods() {
    let patient = discovery::decide_dial("peer-2", "peer-1", Some(2 * discovery::REDIAL_SECS - 1));
    assert_eq!(patient, discovery::DialDecision::Wait);
    let desperate = discovery::decide_dial("peer-2", "peer-1", Some(2 * discovery::REDIAL_SECS));
    assert_eq!(desperate, discovery::DialDecision::ForceDial);
}

#[test]
fn bootstrapped_hints_dedup_across_freenet_and_libp2p() {
    let freenet_hint = discovery::PeerHint {
        peer_id: "peer-2".to_string(),
        addrs: vec!["/ip4/10.0.0.2/tcp/4001".to_string()],
        updated_at: 100,
    };
    let libp2p_hint = discovery::PeerHint {
        peer_id: "peer-2".to_string(),
        addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
        updated_at: 110,
    };
    let merged = discovery::merge_peer_hints(vec![freenet_hint, libp2p_hint]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].updated_at, 110);
}

#[test]
fn star_leaves_still_resolve_the_same_room() {
    let mut mesh = testing::Mesh::three();
    mesh.partition(1, 2);
    let mut directory = directory_with("room-20250101-120000", 100);
    directory = discovery::merge_directory(directory, directory_with("room-20250101-120001", 110));
    let room = discovery::pick_room(&directory, 50).map(|(room, _)| room);
    assert_eq!(room, Some("room-20250101-120001".to_string()));
    mesh.heal(1, 2);
    let counts = mesh.counts();
    assert_eq!(counts.len(), 3);
}
