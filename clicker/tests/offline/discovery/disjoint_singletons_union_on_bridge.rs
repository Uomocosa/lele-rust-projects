use clicker_lib::{discovery, testing};

#[test]
fn disjoint_singletons_union_on_bridge() {
    let mut first = testing::seed_directory("room-20250101-120000", 100);
    let second = testing::seed_directory("room-20250101-120001", 110);
    first = discovery::merge_directory(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(
        discovery::pick_room(&first, 50).map(|(room, _)| room),
        Some("room-20250101-120001".to_string())
    );
}
