use bevy::prelude::Component;

#[derive(Component)]
pub struct IncrementButton;

#[cfg(test)]
mod tests {
    use super::IncrementButton;

    #[test]
    fn test_usage() {
        let _btn = IncrementButton;
    }
}
