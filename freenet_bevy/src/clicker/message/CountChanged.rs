use bevy::prelude::Message;

#[derive(Message, Debug, Clone)]
pub struct CountChanged {
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::CountChanged;

    #[test]
    fn test_usage() {
        let msg = CountChanged { count: 42 };
        assert_eq!(msg.count, 42);
    }
}
