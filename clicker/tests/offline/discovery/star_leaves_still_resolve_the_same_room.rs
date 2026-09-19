use clicker_lib::{discovery, testing};

#[test]
fn star_leaves_still_resolve_the_same_room() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(2).partition_from(3);
    let mut directory = testing::seed_directory("room-20250101-120000", 100);
    directory = discovery::merge_directory(
        directory,
        testing::seed_directory("room-20250101-120001", 110),
    );
    let room = discovery::pick_room(&directory, 50).map(|(room, _)| room);
    assert_eq!(room, Some("room-20250101-120001".to_string()));
    mesh.peer(2).heal_with(3);
    let counts = mesh.counts();
    assert_eq!(counts.len(), 3);
}
