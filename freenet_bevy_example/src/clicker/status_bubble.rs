use bevy::prelude::Component;

#[derive(Component)]
pub struct StatusBubble;

#[cfg(test)]
mod tests {
    use super::StatusBubble;

    #[test]
    fn test_usage() {
        let _bubble = StatusBubble;
    }
}
