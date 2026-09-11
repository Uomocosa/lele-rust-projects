#[must_use]
pub fn math_formatter(count: i32) -> String {
    let sign = if count < 0 { "-" } else { "" };
    let magnitude = i64::from(count).saturating_abs();
    if magnitude < 1_000 {
        return format!("{sign}{magnitude}");
    }
    let mut divisor: i64 = 1_000;
    let mut exp: u32 = 3;
    if magnitude >= 1_000_000_000 {
        divisor = 1_000_000_000;
        exp = 9;
    } else if magnitude >= 1_000_000 {
        divisor = 1_000_000;
        exp = 6;
    }
    let mut tenths = rounded_tenths(magnitude, divisor);
    let mut whole = tenths.checked_div(10).unwrap_or(0);
    while whole >= 1_000 && exp < 9 {
        divisor = divisor.saturating_mul(1_000);
        exp = exp.saturating_add(3);
        tenths = rounded_tenths(magnitude, divisor);
        whole = tenths.checked_div(10).unwrap_or(0);
    }
    let frac = tenths.checked_rem(10).unwrap_or(0);
    if frac == 0 {
        format!("{sign}{whole}e+{exp}")
    } else {
        format!("{sign}{whole}.{frac}e+{exp}")
    }
}

// needed helper: magnitude scaled to tenths of the unit, rounded half up
fn rounded_tenths(magnitude: i64, divisor: i64) -> i64 {
    let half = divisor.checked_div(2).unwrap_or(0);
    magnitude
        .saturating_mul(10)
        .saturating_add(half)
        .checked_div(divisor)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::math_formatter;

    #[test]
    fn test_usage() {
        assert_eq!(math_formatter(0), "0");
        assert_eq!(math_formatter(1), "1");
        assert_eq!(math_formatter(999), "999");
        assert_eq!(math_formatter(1_000), "1e+3");
        assert_eq!(math_formatter(1_050), "1.1e+3");
        assert_eq!(math_formatter(1_100), "1.1e+3");
        assert_eq!(math_formatter(1_234), "1.2e+3");
        assert_eq!(math_formatter(9_999), "10e+3");
        assert_eq!(math_formatter(999_999), "1e+6");
        assert_eq!(math_formatter(1_000_000), "1e+6");
        assert_eq!(math_formatter(1_500_000), "1.5e+6");
        assert_eq!(math_formatter(2_147_483_647), "2.1e+9");
        assert_eq!(math_formatter(-1_500), "-1.5e+3");
    }
}
