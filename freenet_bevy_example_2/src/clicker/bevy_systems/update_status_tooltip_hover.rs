use bevy::prelude::*;

use crate::clicker;

pub fn update_status_tooltip_hover(
    bubble_query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<clicker::StatusBubble>),
    >,
    mut tooltip_query: Query<&mut Visibility, With<clicker::StatusTooltip>>,
) {
    for (interaction, children) in &bubble_query {
        let visible = matches!(interaction, Interaction::Hovered | Interaction::Pressed);
        for child in children.iter() {
            if let Ok(mut visibility) = tooltip_query.get_mut(child) {
                *visibility = if visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::update_status_tooltip_hover;
    use crate::clicker;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let tooltip = app
            .world_mut()
            .spawn((clicker::StatusTooltip, Visibility::Hidden))
            .id();
        let mut bubble = app
            .world_mut()
            .spawn((clicker::StatusBubble, Interaction::Hovered));
        bubble.add_child(tooltip);

        app.add_systems(Update, update_status_tooltip_hover);
        app.update();

        let visibility = app.world().get::<Visibility>(tooltip).unwrap();
        assert_eq!(*visibility, Visibility::Visible);
    }
}
