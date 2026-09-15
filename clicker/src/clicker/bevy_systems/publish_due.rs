use bevy::prelude::*;

use crate::clicker;

#[must_use]
pub fn publish_due(time: Res<Time>, mut last: Local<f64>) -> bool {
    let time = time.into_inner();
    let now = time.elapsed().as_secs_f64();
    if *last > 0.0 && now - *last < clicker::SNAPSHOT_INTERVAL_SECS {
        return false;
    }
    *last = now;
    true
}

#[cfg(test)]
mod tests {
    use super::publish_due;
    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};

    #[derive(Resource, Default, Deref, DerefMut)]
    struct DueProbed(bool);

    // needed helper: records that the gated system ran
    fn probe(mut out: ResMut<DueProbed>) {
        **out = true;
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.init_resource::<DueProbed>();
        app.add_systems(Update, probe.run_if(publish_due));
        app.update();
        assert!(**app.world().resource::<DueProbed>());
    }
}
