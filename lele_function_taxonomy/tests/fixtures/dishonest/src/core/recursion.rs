use std::time::SystemTime;

fn step_a() -> SystemTime {
    step_b()
}

fn step_b() -> SystemTime {
    if false {
        return step_a();
    }
    SystemTime::now()
}

pub fn recursive_clock() -> SystemTime {
    step_a()
}
