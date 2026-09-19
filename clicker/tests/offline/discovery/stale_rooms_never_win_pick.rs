use clicker_lib::{discovery, testing};

#[test]
fn stale_rooms_never_win_pick() {
    let state = testing::seed_directory("room-20250101-120000", 100);
    assert!(discovery::pick_room(&state, 100).is_none());
    assert!(discovery::pick_room(&state, 500).is_none());
}
