/// Asserts a live X display is configured, naming what's missing otherwise.
pub fn assert_live_display() {
    assert!(
        non_blank(std::env::var("DISPLAY").ok()).is_some(),
        "DISPLAY required for this test"
    );
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

// no test_usage necessary
