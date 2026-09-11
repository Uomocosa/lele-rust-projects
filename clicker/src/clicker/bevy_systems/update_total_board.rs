use bevy::prelude::*;

use crate::clicker;

pub fn update_total_board(
    mut commands: Commands,
    boards: Query<(Entity, Option<&Children>), With<clicker::TotalBoard>>,
    global: Res<clicker::GlobalCounter>,
    mut last: Local<Option<i32>>,
) {
    let value = **global.into_inner();
    if *last == Some(value) {
        return;
    }
    *last = Some(value);
    let (leading, significant, suffix) = clicker::odometer_formatter(value);
    let body = format!("{significant}{suffix}");
    tracing::info!("total board rebuild value={value}");
    for (entity, children) in &boards {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(entity).with_children(|parent| {
            if !leading.is_empty() {
                parent.spawn((
                    TextSpan::new(leading.clone()),
                    TextFont {
                        font_size: FontSize::Px(clicker::TOTAL_LEADING_FONT),
                        ..default()
                    },
                ));
            }
            parent.spawn((
                TextSpan::new(body.clone()),
                TextFont {
                    font_size: FontSize::Px(clicker::TOTAL_SIGNIFICANT_FONT),
                    ..default()
                },
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_total_board;
    use crate::clicker;

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
        app.insert_resource(clicker::GlobalCounter(1_234));
        let board = app
            .world_mut()
            .spawn((clicker::TotalBoard, Text2d::new("")))
            .id();
        app.add_systems(Update, update_total_board);
        app.update();
        app.update();
        let children = app.world().get::<Children>(board).unwrap();
        let mut texts = Vec::new();
        for child in children.iter() {
            let span = app.world().get::<TextSpan>(child).unwrap();
            texts.push((**span).clone());
        }
        assert_eq!(texts, vec!["00000".to_owned(), "1234".to_owned()]);
        app.update();
        let children = app.world().get::<Children>(board).unwrap();
        assert_eq!(children.len(), 2);
    }
}
