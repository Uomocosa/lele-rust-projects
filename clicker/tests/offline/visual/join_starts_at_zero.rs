use clicker_lib::{clicker, testing};

#[test]
fn join_starts_at_zero() {
    let mut mesh = testing::Mesh::of(3);
    for _ in 0..3 {
        mesh.step();
    }
    for app in &mut mesh.apps {
        let mut query = app
            .world_mut()
            .query::<(&clicker::ClickCounter, Option<&clicker::PlayerNo>)>();
        let mut seen: u32 = 0;
        for (counter, _player) in query.iter(app.world()) {
            assert_eq!(**counter, 0, "every cursor starts at 0, never 1");
            seen = seen.saturating_add(1);
        }
        assert_eq!(seen, 3, "own plus two remote placeholders");
    }
}
