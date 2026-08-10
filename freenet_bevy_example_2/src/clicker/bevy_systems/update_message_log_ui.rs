use std::collections::VecDeque;
use std::time::Duration;

use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::lens::TextColorLens;
use bevy_tweening::{Tween, TweenAnim};

use crate::clicker;
use crate::clicker::state::LOG_CAPACITY;

const ENTER_DURATION: Duration = Duration::from_millis(250);
const EXIT_DURATION: Duration = Duration::from_millis(200);
const TEXT_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);

pub fn update_message_log_ui(
    mut commands: Commands,
    mut added_reader: MessageReader<clicker::LogMessageAdded>,
    container_query: Query<Entity, With<clicker::LogContainer>>,
    mut rows: Local<VecDeque<Entity>>,
) {
    let Ok(container) = container_query.single() else {
        return;
    };

    for added in added_reader.read() {
        let transparent = TEXT_COLOR.with_alpha(0.0);
        let row = commands
            .spawn((
                Text::new(added.0.display()),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..Default::default()
                },
                TextColor(transparent),
                clicker::LogRow,
            ))
            .id();
        commands.entity(container).add_child(row);

        let enter_tween = Tween::new(
            EaseFunction::QuadraticOut,
            ENTER_DURATION,
            TextColorLens {
                start: transparent,
                end: TEXT_COLOR,
            },
        );
        commands.entity(row).insert(TweenAnim::new(enter_tween));

        rows.push_back(row);

        if rows.len() > LOG_CAPACITY {
            if let Some(oldest) = rows.pop_front() {
                let exit_tween = Tween::new(
                    EaseFunction::QuadraticIn,
                    EXIT_DURATION,
                    TextColorLens {
                        start: TEXT_COLOR,
                        end: transparent,
                    },
                );
                commands.entity(oldest).insert((
                    TweenAnim::new(exit_tween),
                    clicker::PendingDespawn {
                        timer: Timer::new(EXIT_DURATION, TimerMode::Once),
                    },
                ));
            }
        }
    }
}

pub fn despawn_pending(
    mut commands: Commands,
    time: Res<Time>,
    mut pending_query: Query<(Entity, &mut clicker::PendingDespawn)>,
) {
    for (entity, mut pending) in &mut pending_query {
        pending.timer.tick(time.delta());
        if pending.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::update_message_log_ui;
    use crate::clicker;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<clicker::LogMessageAdded>();
        app.world_mut().spawn(clicker::LogContainer);

        app.add_systems(Update, update_message_log_ui);

        app.world_mut()
            .resource_mut::<Messages<clicker::LogMessageAdded>>()
            .write(clicker::LogMessageAdded(clicker::LogMessage::new("hi")));
        app.update();

        let mut query = app.world_mut().query::<&clicker::LogRow>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }
}
