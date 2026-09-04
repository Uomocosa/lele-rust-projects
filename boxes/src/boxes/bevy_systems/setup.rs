#![allow(clippy::missing_const_for_fn)]
use bevy::prelude::*;
pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
#[cfg(test)]
mod tests {
    use super::setup;
    use bevy::prelude::*;
    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Update, setup);
        app.update();
    }
}
