#[must_use]
pub fn status_enabled() -> bool {
    std::env::var("TELEGRAM_NOTIFY_STATUS")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::status_enabled;

    #[test]
    fn test_usage() {
        let _ = status_enabled();
    }
}
