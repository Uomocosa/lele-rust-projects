use bevy::prelude::Component;

/// Marker for a single row entity in the scrolling message log.
#[derive(Component)]
pub struct LogRow;

#[cfg(test)]
mod tests {
    use super::LogRow;

    #[test]
    fn test_usage() {
        let _row = LogRow;
    }
}
