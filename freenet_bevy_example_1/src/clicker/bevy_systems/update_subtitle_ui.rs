use bevy::prelude::*;

use crate::clicker;

pub fn update_subtitle_ui(
    mut connection_reader: MessageReader<clicker::ConnectionChanged>,
    mut subtitle_query: Query<&mut Text, With<clicker::SubtitleText>>,
) {
    for event in connection_reader.read() {
        if let Ok(mut text) = subtitle_query.single_mut() {
            text.0 = if event.connected {
                "Connected to the Freenet network".to_string()
            } else {
                "Connect to a Freenet node to participate".to_string()
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_subtitle_ui;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<clicker::ConnectionChanged>();

        app.world_mut()
            .spawn((clicker::SubtitleText, Text::default()));

        app.add_systems(Update, update_subtitle_ui);

        app.world_mut()
            .resource_mut::<Messages<clicker::ConnectionChanged>>()
            .write(clicker::ConnectionChanged { connected: true });

        app.update();

        let mut query = app.world_mut().query::<&Text>();
        let text = query.single(app.world());
        assert_eq!(
            text.ok().map(|t| t.0.clone()),
            Some("Connected to the Freenet network".to_string())
        );
    }
}
