pub fn record(value: i64) -> i64 {
    fixture_log_stub::info("recording");
    value + 1
}
