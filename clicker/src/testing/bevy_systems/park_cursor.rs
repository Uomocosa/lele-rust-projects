use bevy::prelude::*;

use crate::clicker;

pub fn park_cursor(mut cursors: Query<&mut Transform, With<clicker::CursorIcon>>) {
    for mut cursor in &mut cursors {
        cursor.translation.x = -260.0;
        cursor.translation.y = 120.0;
    }
}

#[cfg(test)]
mod tests {
    use super::park_cursor;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, park_cursor);
        app.update();
    }
}
