use bevy::prelude::Resource;

#[derive(Resource)]
pub struct HeadlessState {
    pub max_ticks: usize,
    pub pending: bool,
    pub completed: usize,
}

#[derive(Clone)]
pub struct HeadlessConfig {
    pub max_ticks: usize,
}

#[cfg(test)]
mod tests {
    use super::{HeadlessConfig, HeadlessState};

    #[test]
    fn test_usage() {
        let config = HeadlessConfig { max_ticks: 5 };
        assert_eq!(config.max_ticks, 5);
        let state = HeadlessState {
            max_ticks: 5,
            pending: false,
            completed: 0,
        };
        assert_eq!(state.max_ticks, 5);
    }
}
