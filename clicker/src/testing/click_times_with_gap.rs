use std::time::Duration;

use bevy::prelude::*;

pub fn click_times_with_gap(app: &mut App, times: u32, gap: Duration) {
    for _ in 0..times {
        let mut input = ButtonInput::<MouseButton>::default();
        input.press(MouseButton::Left);
        app.world_mut().insert_resource(input);
        app.update();
        if !gap.is_zero() {
            std::thread::sleep(gap);
        }
    }
    app.world_mut()
        .insert_resource(ButtonInput::<MouseButton>::default());
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::click_times_with_gap;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::fixture(1, "alpha");
        app.update();
        click_times_with_gap(&mut app, 2, Duration::from_millis(1));
        assert_eq!(testing::get_count(&mut app, 1), 2);
        assert_eq!(testing::get_global(&mut app), 2);
    }
}
