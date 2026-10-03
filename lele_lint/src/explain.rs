use crate::render_rule;
use crate::Checker;

pub fn explain(checkers: &[Box<dyn Checker>], code_or_name: &str) -> Option<String> {
    checkers
        .iter()
        .find(|c| c.code().eq_ignore_ascii_case(code_or_name) || c.name() == code_or_name)
        .map(|c| render_rule(c.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::explain;
    use crate::checkers::build_checkers;

    #[test]
    fn test_usage() {
        let checkers = build_checkers();
        let by_code = explain(&checkers, "e009");
        let by_name = explain(&checkers, "no_positional");
        assert!(by_code.is_some());
        assert_eq!(by_code, by_name);
        assert!(explain(&checkers, "E999").is_none());
    }
}
