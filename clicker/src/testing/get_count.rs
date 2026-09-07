use bevy::prelude::*;

use crate::clicker;

#[must_use]
pub fn get_count(app: &mut App, owner: u64) -> i32 {
    let mut count = 0;
    let mut query = app
        .world_mut()
        .query::<(&clicker::Owner, &clicker::ClickCounter)>();
    for (id, counter) in query.iter(app.world()) {
        if ***id == owner {
            count = **counter;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::get_count;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::fixture(1, "alpha");
        app.update();
        assert_eq!(get_count(&mut app, 1), 0);
    }
}
