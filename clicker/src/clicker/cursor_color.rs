use bevy::prelude::{Color, Component};
use derive_more::Deref;

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct CursorColor(pub Color);

#[cfg(test)]
mod tests {
    use super::CursorColor;
    use bevy::prelude::Color;

    #[test]
    fn test_usage() {
        let base = CursorColor(Color::srgb(0.5, 0.5, 0.5));
        assert_eq!(*base, Color::srgb(0.5, 0.5, 0.5));
    }
}
