use bevy::prelude::*;

pub const fn handle_click(_input: Res<ButtonInput<MouseButton>>) {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
// no test_usage necessary
