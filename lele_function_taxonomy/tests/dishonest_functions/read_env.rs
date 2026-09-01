pub fn read_env() -> String {
    std::env::var("HOME").unwrap_or_default()
}
