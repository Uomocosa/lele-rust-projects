pub fn substitute(args: &[String], vars: &[(&str, String)]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            vars.iter().fold(arg.clone(), |acc, (name, value)| {
                acc.replace(&format!("{{{name}}}"), value)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::substitute;

    #[test]
    fn test_usage() {
        let args = vec!["--bind".to_string(), "127.0.0.1:{port}".to_string()];
        let out = substitute(&args, &[("port", "8123".to_string())]);
        assert_eq!(out, vec!["--bind", "127.0.0.1:8123"]);
    }
}
