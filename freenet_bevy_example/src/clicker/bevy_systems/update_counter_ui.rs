use bevy::prelude::*;

use crate::clicker;

pub fn update_counter_ui(
    mut count_reader: MessageReader<clicker::CountChanged>,
    mut counter_query: Query<&mut Text, With<clicker::CounterText>>,
) {
    for event in count_reader.read() {
        if let Ok(mut text) = counter_query.single_mut() {
            text.0 = event.count.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_counter_ui;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<clicker::CountChanged>();

        app.world_mut()
            .spawn((clicker::CounterText, Text::default()));

        app.add_systems(Update, update_counter_ui);

        app.world_mut()
            .resource_mut::<Messages<clicker::CountChanged>>()
            .write(clicker::CountChanged { count: 99 });

        app.update();

        let mut query = app.world_mut().query::<&Text>();
        let text = query.single(app.world());
        assert_eq!(text.ok().map(|t| t.0.clone()), Some("99".to_string()));
    }
}
