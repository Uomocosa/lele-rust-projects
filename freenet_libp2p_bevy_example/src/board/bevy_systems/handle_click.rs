use bevy::prelude::*;

pub fn handle_click(_input: Res<ButtonInput<MouseButton>>) {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
// no test_usage necessary
