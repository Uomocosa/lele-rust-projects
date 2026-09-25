#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::Multiplayer;
use super::super::multiplayer_feed::MultiplayerFeed;

pub fn drain_multiplayer(feed: Res<MultiplayerFeed>, mut multiplayer: ResMut<Multiplayer>) {
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let mut latest: Option<Multiplayer> = None;
    while let Ok(snapshot) = rx.try_recv() {
        latest = Some(snapshot);
    }
    if let Some(snapshot) = latest {
        *multiplayer = snapshot;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bevy::prelude::*;

    use super::drain_multiplayer;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Multiplayer>();
        app.insert_resource(discovery::MultiplayerFeed(Mutex::new(Some(rx))));
        app.add_systems(Update, drain_multiplayer);
        tx.send(discovery::Multiplayer::default()).ok();
        app.update();
        assert!(
            app.world()
                .resource::<discovery::Multiplayer>()
                .room
                .is_none()
        );
    }
}
