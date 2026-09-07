use bevy::prelude::*;

pub fn click_times(app: &mut App, times: u32) {
    for _ in 0..times {
        let mut input = ButtonInput::<MouseButton>::default();
        input.press(MouseButton::Left);
        app.world_mut().insert_resource(input);
        app.update();
    }
    app.world_mut()
        .insert_resource(ButtonInput::<MouseButton>::default());
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
