pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn scale(values: &[i64], factor: i64) -> Vec<i64> {
    values.iter().map(|v| v * factor).collect()
}
