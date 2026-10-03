pub fn squared(n: i64) -> i64 {
    n * n
}

pub fn sum_of_squares(values: &[i64]) -> i64 {
    values.iter().map(|v| squared(*v)).sum()
}
