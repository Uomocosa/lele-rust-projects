use atomic_delegate_macros::atomic_delegates;

pub struct Widget {
    pub id: u32,
}

impl Default for Widget {
    fn default() -> Self {
        Self { id: 0 }
    }
}

#[atomic_delegates]
impl Widget {
    pub fn stamp(&self) -> u64 {}
}
