use bevy::prelude::Component;
use derive_more::{Deref, DerefMut};

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct ClickFlash(pub f32);

#[cfg(test)]
mod tests {
    use super::ClickFlash;

    #[test]
    fn test_usage() {
        let mut flash = ClickFlash::default();
        assert_eq!(*flash, 0_f32);
        *flash = 0.2;
        assert_eq!(*flash, 0.2);
    }
}
