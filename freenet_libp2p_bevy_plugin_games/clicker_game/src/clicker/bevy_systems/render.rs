use bevy::prelude::*;

use crate::clicker;

#[derive(Component)]
struct ScoreboardText;

pub fn render(
    mut commands: Commands,
    counters: Query<(&clicker::Owner, &clicker::ClickCounter)>,
    mut text_entity: Local<Option<Entity>>,
) {
    let mut lines: Vec<String> = Vec::new();
    for (owner, counter) in &counters {
        lines.push(format!("{}: {}", ***owner, **counter));
    }
    let text = if lines.is_empty() {
        "no players".to_string()
    } else {
        lines.join("\n")
    };
    match *text_entity {
        Some(entity) => {
            commands.entity(entity).insert(Text::new(text));
        }
        None => {
            let entity = commands
                .spawn((
                    ScoreboardText,
                    Text::new(text),
                    TextFont {
                        font_size: FontSize::Px(20.0),
                        ..Default::default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(8.0),
                        top: Val::Px(8.0),
                        ..Default::default()
                    },
                ))
                .id();
            *text_entity = Some(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::render;
    use crate::clicker;
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

        let mut texts = app.world_mut().query::<&Text>();
        let rendered: Vec<String> = texts
            .iter(app.world())
            .map(|text| match text {
                Text(content) => content.clone(),
            })
            .collect();
        assert_eq!(rendered, vec!["7: 4".to_string()]);
    }
}
