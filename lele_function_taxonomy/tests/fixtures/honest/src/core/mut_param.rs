pub fn double_all(values: &mut [i64]) {
    for value in values.iter_mut() {
        *value *= 2;
    }
}
