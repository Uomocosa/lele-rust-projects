use crate::clicker;
use bevy::prelude::*;
pub const fn render(_counters: Query<(&clicker::Owner, &clicker::ClickCounter)>) {}
#[cfg(test)]
mod tests {
    use super::render;
    use crate::clicker;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::net_id;
    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(7)),
            clicker::ClickCounter(4),
        ));
        app.add_systems(Update, render);
        app.update();
    }
}
