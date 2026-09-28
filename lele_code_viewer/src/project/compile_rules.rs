use regex::Regex;

pub fn compile_rules(rules: &[String]) -> Result<Vec<Regex>, String> {
    let mut compiled = Vec::new();
    for (line, rule) in rules.iter().enumerate() {
        let rule = rule.trim();
        if rule.is_empty() {
            continue;
        }
        let regex = Regex::new(rule).map_err(|err| {
            format!(
                "rule {} `{rule}` is not a valid regex: {err}",
                line.saturating_add(1)
            )
        })?;
        compiled.push(regex);
    }
    Ok(compiled)
}

#[cfg(test)]
mod tests {
    use super::compile_rules;
    use crate::project;

    #[test]
    fn test_usage() {
        let rules = project::default_settings().ignore;
        let compiled = compile_rules(&rules).unwrap();
        assert_eq!(compiled.len(), 4);
        let hit = |path: &str| compiled.iter().any(|rule| rule.is_match(path));
        assert!(hit("/home/u/proj/target"));
        assert!(hit("/home/u/proj/.git/hooks"));
        assert!(hit("/home/u/rust/__OLD__/crate"));
        assert!(!hit("/home/u/my_target_app"));
        assert!(!hit("/home/u/proj/.github"));
    }

    #[test]
    fn test_invalid_rule_is_reported_with_its_line() {
        let rules = vec!["ok".to_string(), String::new(), "(unclosed".to_string()];
        let err = compile_rules(&rules).unwrap_err();
        assert!(err.starts_with("rule 3 `(unclosed`"));
    }
}
