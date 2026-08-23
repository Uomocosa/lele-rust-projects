use bevy::prelude::*;

use crate::roster;

#[derive(Component)]
struct FreenetStatusText;

pub fn update_status_text(
    mut commands: Commands,
    status: Res<roster::FreenetStatus>,
    mut text_entity: Local<Option<Entity>>,
) {
    let text = match *status {
        roster::FreenetStatus::Connecting => "freenet: connecting...".to_string(),
        roster::FreenetStatus::Connected => "freenet: connected".to_string(),
        roster::FreenetStatus::Retrying { attempt } => {
            format!("freenet: connecting... retry {attempt}")
        }
    };
    match *text_entity {
        Some(entity) => {
            commands.entity(entity).insert(Text::new(text));
        }
        None => {
            let entity = commands
                .spawn((
                    FreenetStatusText,
                    Text::new(text),
                    TextFont {
                        font_size: FontSize::Px(16.0),
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

    use super::update_status_text;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(roster::FreenetStatus::default());
        app.add_systems(Update, update_status_text);
        app.update();

        app.insert_resource(roster::FreenetStatus::Connected);
        app.update();

        let mut texts = app.world_mut().query::<&Text>();
        let rendered: Vec<String> = texts
            .iter(app.world())
            .map(|text| match text {
                Text(content) => content.clone(),
            })
            .collect();
        assert_eq!(rendered, vec!["freenet: connected".to_string()]);
    }
}
