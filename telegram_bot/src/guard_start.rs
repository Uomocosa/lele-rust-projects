use super::guard::Guard;

#[must_use]
pub fn start(name: &str) -> Guard {
    Guard {
        name: name.to_string(),
        start: std::time::Instant::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::start;

    #[test]
    fn test_usage() {
        let guard = start("guard_start_smoke");
        assert_eq!(guard.name, "guard_start_smoke");
    }
}
