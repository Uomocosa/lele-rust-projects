use bevy::prelude::*;

use crate::clicker::component::CounterText::CounterText;
use crate::clicker::message::CountChanged::CountChanged;

pub fn update_counter_ui(
    mut count_reader: MessageReader<CountChanged>,
    mut counter_query: Query<&mut Text, With<CounterText>>,
) {
    for event in count_reader.read() {
        if let Ok(mut text) = counter_query.single_mut() {
            text.0 = event.count.to_string();
        }
    }
}
