use std::time::Duration;

use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::lens::UiBackgroundColorLens;
use bevy_tweening::{Tween, TweenAnim};

use crate::clicker;

const GREEN: Color = Color::srgb(0.2, 0.8, 0.2);
const ORANGE: Color = Color::srgb(0.9, 0.6, 0.1);
const RED: Color = Color::srgb(0.85, 0.2, 0.2);

fn status_color(status: &clicker::ConnectionStatus) -> Color {
    match status {
        clicker::ConnectionStatus::Connecting => ORANGE,
        clicker::ConnectionStatus::Connected => GREEN,
        clicker::ConnectionStatus::Error(_) => RED,
    }
}

fn status_tooltip_text(status: &clicker::ConnectionStatus) -> String {
    match status {
        clicker::ConnectionStatus::Connecting => "connecting...".to_string(),
        clicker::ConnectionStatus::Connected => "connected".to_string(),
        clicker::ConnectionStatus::Error(_) => {
            "could not connect to the freenet nodes. Try to close and reopen the app and check your internet connection.".to_string()
        }
    }
}

pub fn update_status_bubble_ui(
    mut commands: Commands,
    mut connection_reader: MessageReader<clicker::ConnectionChanged>,
    bubble_query: Query<
        (Entity, &BackgroundColor, &Children),
        With<clicker::StatusBubble>,
    >,
    mut tooltip_text_query: Query<&mut Text, With<clicker::StatusTooltip>>,
    tooltip_children_query: Query<&Children, With<clicker::StatusBubble>>,
) {
    for event in connection_reader.read() {
        let target = status_color(&event.status);
        let tooltip = status_tooltip_text(&event.status);

        for (entity, bg, children) in &bubble_query {
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(300),
                UiBackgroundColorLens {
                    start: bg.0,
                    end: target,
                },
            );
            commands.entity(entity).insert(TweenAnim::new(tween));

            for child in children.iter() {
                if let Ok(mut text) = tooltip_text_query.get_mut(child) {
                    text.0 = tooltip.clone();
                }
            }
            let _ = &tooltip_children_query;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{status_color, status_tooltip_text};
    use crate::clicker::ConnectionStatus;

    #[test]
    fn test_usage() {
        assert!(status_tooltip_text(&ConnectionStatus::Connected).contains("connected"));
        assert!(status_tooltip_text(&ConnectionStatus::Error("x".into())).contains("could not connect"));
        let _ = status_color(&ConnectionStatus::Connecting);
    }
}
