pub fn apply_twice<F: Fn(i64) -> i64>(f: F, value: i64) -> i64 {
    f(f(value))
}
