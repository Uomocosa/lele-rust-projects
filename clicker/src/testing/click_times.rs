use std::time::Duration;

use bevy::prelude::*;

use crate::testing;

pub fn click_times(app: &mut App, times: u32) {
    testing::click_times_with_gap(app, times, Duration::ZERO);
}

#[cfg(test)]
mod tests {
    use super::click_times;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::fixture(1, "alpha");
        app.update();
        click_times(&mut app, 3);
        assert_eq!(testing::get_count(&mut app, 1), 3);
        assert_eq!(testing::get_global(&mut app), 3);
    }
}
