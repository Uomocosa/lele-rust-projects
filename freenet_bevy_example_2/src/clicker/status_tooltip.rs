use bevy::prelude::Component;

/// Marker for the hover tooltip text spawned next to the status bubble.
#[derive(Component)]
pub struct StatusTooltip;

#[cfg(test)]
mod tests {
    use super::StatusTooltip;

    #[test]
    fn test_usage() {
        let _tooltip = StatusTooltip;
    }
}
