use bevy::prelude::*;

use crate::clicker;

pub fn render(
    mut query: Query<(&clicker::ClickCounter, &mut Transform), With<clicker::ClickTarget>>,
) {
    for (counter, mut transform) in &mut query {
        let clamped = i16::try_from(**counter).unwrap_or(i16::MAX);
        let growth = 0.02f32.mul_add(f32::from(clamped), 1.0);
        transform.scale = Vec3::splat(growth);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::render;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let target = app
            .world_mut()
            .spawn((
                clicker::ClickCounter(10),
                clicker::ClickTarget,
                Transform::default(),
            ))
            .id();
        app.add_systems(Update, render);
        app.update();
        let scale = app.world().get::<Transform>(target).unwrap().scale;
        assert!(scale.x > 1.0);
    }
}
