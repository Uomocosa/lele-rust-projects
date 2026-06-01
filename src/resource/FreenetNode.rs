use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct FreenetNode {
    pub shutdown_handle: Option<freenet::ShutdownHandle>,
}
