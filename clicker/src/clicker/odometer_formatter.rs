#[must_use]
pub fn odometer_formatter(count: i32) -> (String, String, String) {
    const SCALE: i64 = 1_000;
    const THRESHOLD: i64 = 1_000_000_000;
    const WIDTH: usize = 9;
    let magnitude = i64::from(count).max(0);
    let (mantissa, suffix) = if magnitude >= THRESHOLD {
        (magnitude.checked_div(SCALE).unwrap_or(0), "e3")
    } else {
        (magnitude, "")
    };
    let digits = format!("{mantissa:09}");
    let trimmed = digits.trim_start_matches('0');
    let significant = if trimmed.is_empty() { "0" } else { trimmed };
    let zeros = digits.len().saturating_sub(significant.len()).min(WIDTH);
    ("0".repeat(zeros), significant.to_owned(), suffix.to_owned())
}

#[cfg(test)]
mod tests {
    use super::odometer_formatter;

    #[test]
    fn test_usage() {
        assert_eq!(
            odometer_formatter(0),
            ("00000000".to_owned(), "0".to_owned(), String::new())
        );
        assert_eq!(
            odometer_formatter(1),
            ("00000000".to_owned(), "1".to_owned(), String::new())
        );
        assert_eq!(
            odometer_formatter(999),
            ("000000".to_owned(), "999".to_owned(), String::new())
        );
        assert_eq!(
            odometer_formatter(999_999_999),
            (String::new(), "999999999".to_owned(), String::new())
        );
        assert_eq!(
            odometer_formatter(1_000_000_000),
            ("00".to_owned(), "1000000".to_owned(), "e3".to_owned())
        );
        assert_eq!(
            odometer_formatter(2_147_483_647),
            ("00".to_owned(), "2147483".to_owned(), "e3".to_owned())
        );
        assert_eq!(
            odometer_formatter(-5),
            ("00000000".to_owned(), "0".to_owned(), String::new())
        );
    }
}
