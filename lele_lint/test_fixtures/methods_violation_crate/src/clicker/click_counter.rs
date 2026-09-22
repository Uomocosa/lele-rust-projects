use derive_more::Deref;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deref)]
pub struct ClickCounter(pub i32);

// this comment is banned by E031
impl ClickCounter {
    pub fn add(&mut self, _delta: i32) {}
}

#[atomic_delegate]
impl ClickCounter {
    pub fn increment(&mut self) {}
}
