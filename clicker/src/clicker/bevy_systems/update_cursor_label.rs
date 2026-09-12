use bevy::prelude::*;

use crate::clicker;

pub fn update_cursor_label(
    targets: Query<(&clicker::ClickCounter, &Children)>,
    mut labels: Query<&mut Text2d, With<clicker::CursorLabel>>,
) {
    for (counter, children) in &targets {
        let text = clicker::math_formatter(**counter);
        for child in children {
            if let Ok(mut label) = labels.get_mut(*child)
                && label.0 != text
            {
                tracing::debug!("cursor label change {} -> {text}", label.0);
                (**label).clone_from(&text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_cursor_label;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    // needed helper: enables debug logs for this test only
    fn test_logging(app: &mut App) {
        app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: "info,clicker_lib=debug".to_string(),
            ..default()
        });
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        test_logging(&mut app);
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(1)),
                clicker::ClickCounter(1_234),
            ))
            .with_children(|parent| {
                parent.spawn((clicker::CursorLabel, Text2d::new("0")));
            });
        app.add_systems(Update, update_cursor_label);
        app.update();
        let mut found = false;
        let mut query = app.world_mut().query::<&Text2d>();
        for text in query.iter(app.world()) {
            if text.0 == "1.2e+3" {
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn each_label_shows_its_own_cursor_count() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        test_logging(&mut app);
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(1)),
                clicker::ClickCounter(3),
            ))
            .with_children(|parent| {
                parent.spawn((clicker::CursorLabel, Text2d::new("0")));
            });
        app.world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(2)),
                clicker::ClickCounter(17),
            ))
            .with_children(|parent| {
                parent.spawn((clicker::CursorLabel, Text2d::new("0")));
            });
        app.add_systems(Update, update_cursor_label);
        app.update();
        let mut texts = Vec::new();
        let mut query = app.world_mut().query::<&Text2d>();
        for text in query.iter(app.world()) {
            texts.push(text.0.clone());
        }
        texts.sort();
        assert_eq!(texts, vec!["17".to_string(), "3".to_string()]);
    }
}
