use bevy::prelude::Component;

/// Marker for the parent node that holds the (up to 5) log row entities.
#[derive(Component)]
pub struct LogContainer;

#[cfg(test)]
mod tests {
    use super::LogContainer;

    #[test]
    fn test_usage() {
        let _container = LogContainer;
    }
}
