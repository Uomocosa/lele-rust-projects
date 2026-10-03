pub trait Clock {
    fn now(&self) -> u64;
}

pub fn report<C: Clock>(clock: &C) -> u64 {
    clock.now()
}

pub fn report_dyn(clock: &dyn Clock) -> u64 {
    clock.now()
}
