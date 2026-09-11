use bevy::prelude::*;

use crate::clicker;

pub fn emit_flash(mut commands: Commands, changed: Query<Entity, Changed<clicker::ClickCounter>>) {
    for entity in &changed {
        commands
            .entity(entity)
            .insert(clicker::ClickFlash(clicker::FLASH_SECS));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::emit_flash;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let target = app.world_mut().spawn(clicker::ClickCounter(3)).id();
        app.add_systems(Update, emit_flash);
        app.update();
        let flash = app.world().get::<clicker::ClickFlash>(target).unwrap();
        assert_eq!(**flash, clicker::FLASH_SECS);
        app.update();
        assert!(app.world().get::<clicker::ClickFlash>(target).is_some());
    }
}
