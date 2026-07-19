use bevy::prelude::Message;

#[derive(Message, Debug, Clone)]
pub struct CountChanged {
    pub count: u64,
}
