use bevy::prelude::*;

use crate::clicker;

pub fn tick_join_countdown(
    query: Query<(&clicker::PendingReveal, &Children)>,
    mut labels: Query<&mut Text2d, With<clicker::CursorLabel>>,
) {
    for (reveal, children) in &query {
        let deadline = if reveal.player.is_some() {
            reveal.reveal_at
        } else {
            reveal.fail_at
        };
        let millis = deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_millis();
        let secs = u64::try_from(millis)
            .unwrap_or(u64::MAX)
            .saturating_add(999)
            .checked_div(1_000)
            .unwrap_or(0);
        let text = format!("{secs}s");
        for child in children {
            if let Ok(mut label) = labels.get_mut(*child)
                && label.0 != text
            {
                (**label).clone_from(&text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::tick_join_countdown;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let now = std::time::Instant::now();
        let reveal_at = now
            .checked_add(std::time::Duration::from_millis(2_500))
            .unwrap_or(now);
        let cursor = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::PendingReveal {
                    reveal_at,
                    fail_at: now,
                    player: Some(2),
                },
            ))
            .id();
        let label = app
            .world_mut()
            .spawn((clicker::CursorLabel, Text2d::new("0")))
            .id();
        app.world_mut().entity_mut(cursor).add_child(label);
        app.add_systems(Update, tick_join_countdown);
        app.update();
        let text = app.world().get::<Text2d>(label).map(|t| t.0.clone());
        assert_eq!(text, Some("3s".to_string()), "counts down by whole seconds");
    }

    #[test]
    fn unresolved_counts_to_fail_deadline() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let now = std::time::Instant::now();
        let fail_at = now
            .checked_add(std::time::Duration::from_millis(4_500))
            .unwrap_or(now);
        let cursor = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::PendingReveal {
                    reveal_at: now,
                    fail_at,
                    player: None,
                },
            ))
            .id();
        let label = app
            .world_mut()
            .spawn((clicker::CursorLabel, Text2d::new("0")))
            .id();
        app.world_mut().entity_mut(cursor).add_child(label);
        app.add_systems(Update, tick_join_countdown);
        app.update();
        let text = app.world().get::<Text2d>(label).map(|t| t.0.clone());
        assert_eq!(text, Some("5s".to_string()));
    }
}
