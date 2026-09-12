use bevy::prelude::*;

use crate::clicker;

pub fn sync_global(
    targets: Query<&clicker::ClickCounter>,
    mut global: ResMut<clicker::GlobalCounter>,
) {
    let mut total: i32 = 0;
    for counter in &targets {
        total = total.saturating_add(**counter);
    }
    if **global != total {
        **global = total;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_global;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::GlobalCounter(999));
        app.world_mut().spawn(clicker::ClickCounter(4));
        app.world_mut().spawn(clicker::ClickCounter(5));
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 9);
    }
}
