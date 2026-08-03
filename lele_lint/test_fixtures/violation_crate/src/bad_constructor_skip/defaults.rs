pub struct Defaults {
    pub timeout: u64,
    pub retries: u32,
}

#[rustfmt::skip]                          // VIOLATION: Default should NOT be rustfmt::skip
impl Default for Defaults {
    fn default() -> Self {
        Self {
            timeout: 30,
            retries: 3,
        }
    }
}

#[rustfmt::skip]                          // VIOLATION: constructor with selfless New type
impl Defaults {
    pub fn production() -> Self {
        Self {
            timeout: 5,
            retries: 1,
        }
    }
}
