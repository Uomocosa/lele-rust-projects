pub static NAMES: [&str; 2] = ["alpha", "beta"];

pub fn first_name() -> &'static str {
    NAMES[0]
}

pub fn name_count() -> usize {
    NAMES.len()
}
