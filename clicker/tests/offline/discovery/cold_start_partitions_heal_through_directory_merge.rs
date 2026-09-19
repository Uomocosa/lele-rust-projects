use clicker_lib::{discovery, testing};

#[test]
fn cold_start_partitions_heal_through_directory_merge() {
    let singleton_a = testing::seed_directory("room-20250101-120000", 100);
    let singleton_b = testing::seed_directory("room-20250101-120000", 100);
    let merged = discovery::merge_directory(singleton_a, singleton_b);
    assert_eq!(merged.len(), 1);
    assert_eq!(
        discovery::pick_room(&merged, 50).map(|(room, _)| room),
        Some("room-20250101-120000".to_string())
    );
}
