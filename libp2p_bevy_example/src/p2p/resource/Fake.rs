use bevy::prelude::Resource;

use crate::p2p::resource::FakeMethod;

#[derive(Resource)]
pub struct Fake {
    pub enabled: bool,
}

impl Default for Fake {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[rustfmt::skip]
impl Fake {
    pub fn new() -> Self { FakeMethod::new() }
    pub fn disable(self) -> Self { FakeMethod::disable(self) }
}
