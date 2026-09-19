use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::net_id;
use telegram_bot::TestLog;

#[test]
#[telegram_bot::telegram_notify]
fn resolved_cursors_match_owner_color() {
    let test_log = TestLog::open("resolved_cursors_match_owner_color");
    test_log.line("test started");
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-2", 2));
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-3", 3));
    for _ in 0..3 {
        mesh.step();
    }
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let mut seen: u32 = 0;
            let mut query = app
                .world_mut()
                .query::<(&clicker::PlayerNo, &clicker::CursorColor)>();
            for (player, color) in query.iter(app.world()) {
                let want = clicker::color_for(net_id::NetworkId(**player));
                assert_eq!(**color, want, "player {} color", **player);
                seen = seen.saturating_add(1);
            }
            assert_eq!(seen, 2, "two resolved remote cursors");
        }
    }
}
