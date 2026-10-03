use std::time::SystemTime as Clock;

pub fn now_aliased() -> Clock {
    Clock::now()
}
