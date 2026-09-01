use std::time::SystemTime;

pub fn get_time_inner() -> SystemTime {
    SystemTime::now()
}

pub fn caller_calls_dishonest() -> SystemTime {
    get_time_inner()
}
