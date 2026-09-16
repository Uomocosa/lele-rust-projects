use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;
use crate::lobby;

pub fn detect_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut targets: Query<(&clicker::Owner, &mut clicker::ClickCounter)>,
    mut global: ResMut<clicker::GlobalCounter>,
    mut commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
    pending: Res<lobby::JoinPending>,
) {
    let mouse = mouse.into_inner();
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let pending = pending.into_inner();
    if pending.is_some() {
        return;
    }
    tracing::debug!("click anywhere");
    let own = *own.into_inner();
    let lobby = lobby.into_inner();
    for (owner, mut counter) in &mut targets {
        if **owner == own {
            clicker::click(&mut counter, &mut global, &mut commands, lobby, own);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::detect_click;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        app.insert_resource(mouse);
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending::default());
        let target = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(1)),
                clicker::ClickCounter::default(),
            ))
            .id();
        app.add_systems(Update, detect_click);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            1
        );
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 1);
    }

    #[test]
    fn test_no_press_no_click() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending::default());
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        app.add_systems(Update, detect_click);
        app.update();
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 0);
    }

    #[test]
    fn click_dropped_while_join_pending() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        app.insert_resource(mouse);
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending(Some("alpha".to_string())));
        let target = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(1)),
                clicker::ClickCounter::default(),
            ))
            .id();
        app.add_systems(Update, detect_click);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            0,
            "loading joiner registers no clicks"
        );
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 0);
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            0
        );
    }
}
