use bevy::prelude::*;

use crate::clicker;

#[must_use]
pub fn get_global(app: &mut App) -> i32 {
    **app.world_mut().resource::<clicker::GlobalCounter>()
}

#[cfg(test)]
mod tests {
    use super::get_global;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::fixture(1, "alpha");
        app.update();
        assert_eq!(get_global(&mut app), 0);
    }
}
