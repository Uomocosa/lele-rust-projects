#![allow(clippy::missing_const_for_fn)]
use bevy::prelude::*;
pub fn read_input(_input: Res<ButtonInput<KeyCode>>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
