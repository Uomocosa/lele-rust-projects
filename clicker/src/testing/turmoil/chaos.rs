use std::time::Duration;

pub enum Chaos {
    Calm,
    Leave {
        who: &'static str,
        at: Duration,
    },
    Flaky {
        a: &'static str,
        b: &'static str,
        cycles: u32,
    },
}
