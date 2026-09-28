use crate::report;

pub fn state_status(
    previous: Option<&report::Manifest>,
    state: &report::StateRecord,
) -> &'static str {
    let Some(previous) = previous else {
        return report::STATUS_FIRST_RUN;
    };
    let before = previous
        .states
        .iter()
        .find(|old| old.group == state.group && old.fingerprint == state.fingerprint);
    match before {
        None => report::STATUS_NEW,
        Some(old) if old.pixel_hash != state.pixel_hash => report::STATUS_CHANGED,
        Some(_) => report::STATUS_SAME,
    }
}

#[cfg(test)]
mod tests {
    use super::state_status;
    use crate::report;

    fn record(fingerprint: &str, pixel_hash: &str) -> report::StateRecord {
        report::StateRecord {
            id: "s000".to_string(),
            group: "mobile".to_string(),
            screen: "/".to_string(),
            path: Vec::new(),
            location: "/".to_string(),
            png: String::new(),
            fingerprint: fingerprint.to_string(),
            pixel_hash: pixel_hash.to_string(),
        }
    }

    #[test]
    fn test_usage() {
        let previous = report::Manifest {
            states: vec![record("a", "1")],
            ..report::Manifest::default()
        };
        assert_eq!(
            state_status(None, &record("a", "1")),
            report::STATUS_FIRST_RUN
        );
        assert_eq!(
            state_status(Some(&previous), &record("a", "1")),
            report::STATUS_SAME
        );
        assert_eq!(
            state_status(Some(&previous), &record("a", "2")),
            report::STATUS_CHANGED
        );
        assert_eq!(
            state_status(Some(&previous), &record("b", "1")),
            report::STATUS_NEW
        );
    }
}
